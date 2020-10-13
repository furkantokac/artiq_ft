let
  zynq-rs = (import ./zynq-rs.nix);
  pkgs = import <nixpkgs> { overlays = [ (import "${zynq-rs}/nix/mozilla-overlay.nix") ]; };
  rustPlatform = (import "${zynq-rs}/nix/rust-platform.nix" { inherit pkgs; });
  cargo-xbuild = (import zynq-rs).cargo-xbuild;
  zc706-szl = (import zynq-rs).zc706-szl;
  zc706-fsbl = import "${zynq-rs}/nix/fsbl.nix" { inherit pkgs; };
  mkbootimage = import "${zynq-rs}/nix/mkbootimage.nix" { inherit pkgs; };
  artiqpkgs = import <artiq-fast/default.nix> { inherit pkgs; };
  vivado = import <artiq-fast/vivado.nix> { inherit pkgs; };
  build-zc706 = { variant }: let
    firmware = rustPlatform.buildRustPackage rec {
      # note: due to fetchCargoTarball, cargoSha256 depends on package name
      name = "zc706-firmware";

      src = ./src;
      cargoSha256 = "0hjpxqz9ilr4fxi3w3xswn9dcrsh3g2m42vig80xdkpb7wn9gvs0";

      nativeBuildInputs = [
        pkgs.gnumake
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
        cargo-xbuild
        pkgs.llvmPackages_9.llvm
        pkgs.llvmPackages_9.clang-unwrapped
      ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/library"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        make VARIANT=${variant}
      '';

      installPhase = ''
        mkdir -p $out $out/nix-support
        cp ../build/runtime.bin $out/runtime.bin
        cp ../build/firmware/armv7-none-eabihf/release/runtime $out/runtime.elf
        echo file binary-dist $out/runtime.bin >> $out/nix-support/hydra-build-products
        echo file binary-dist $out/runtime.elf >> $out/nix-support/hydra-build-products
      '';

      doCheck = false;
      dontFixup = true;
    };
    gateware = pkgs.runCommand "zc706-${variant}-gateware"
      {
        nativeBuildInputs = [ 
          (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
          vivado
        ];
      }
      ''
        python ${./src/gateware}/zc706.py -g build -V ${variant}
        mkdir -p $out $out/nix-support
        cp build/top.bit $out
        echo file binary-dist $out/top.bit >> $out/nix-support/hydra-build-products
      '';

    # SZL startup
    jtag = pkgs.runCommand "zc706-${variant}-jtag" {}
      ''
        mkdir $out
        ln -s ${zc706-szl}/szl.elf $out
        ln -s ${firmware}/runtime.bin $out
        ln -s ${gateware}/top.bit $out
      '';
    sd = pkgs.runCommand "zc706-${variant}-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      # Do not use "long" paths in boot.bif, because embedded developers
      # can't write software (mkbootimage will segfault).
      bifdir=`mktemp -d`
      cd $bifdir
      ln -s ${zc706-szl}/szl.elf szl.elf
      ln -s ${firmware}/runtime.elf runtime.elf
      ln -s ${gateware}/top.bit top.bit
      cat > boot.bif << EOF
      the_ROM_image:
      {
        [bootloader]szl.elf
        top.bit
        runtime.elf
      }
      EOF
      mkdir $out $out/nix-support
      mkbootimage boot.bif $out/boot.bin
      echo file binary-dist $out/boot.bin >> $out/nix-support/hydra-build-products
      '';

    # FSBL startup
    fsbl-sd = pkgs.runCommand "zc706-${variant}-fsbl-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      bifdir=`mktemp -d`
      cd $bifdir
      ln -s ${zc706-fsbl}/fsbl.elf fsbl.elf
      ln -s ${gateware}/top.bit top.bit
      ln -s ${firmware}/runtime.elf runtime.elf
      cat > boot.bif << EOF
      the_ROM_image:
      {
        [bootloader]fsbl.elf
        top.bit
        runtime.elf
      }
      EOF
      mkdir $out $out/nix-support
      mkbootimage boot.bif $out/boot.bin
      echo file binary-dist $out/boot.bin >> $out/nix-support/hydra-build-products
      '';
  in {
    "zc706-${variant}-firmware" = firmware;
    "zc706-${variant}-gateware" = gateware;
    "zc706-${variant}-jtag" = jtag;
    "zc706-${variant}-sd" = sd;
    "zc706-${variant}-fsbl-sd" = fsbl-sd;
  };
in
  (
    (build-zc706 { variant = "simple"; }) //
    (build-zc706 { variant = "nist_clock"; }) //
    (build-zc706 { variant = "nist_qc2"; }) //
    (build-zc706 { variant = "acpki_simple"; }) //
    (build-zc706 { variant = "acpki_nist_clock"; }) //
    (build-zc706 { variant = "acpki_nist_qc2"; }) //
    { inherit zynq-rs; }
  )
