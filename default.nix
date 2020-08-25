{
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rustPlatform = (import ./rustPlatform.nix { inherit pkgs; });
  artiqpkgs = import <artiq-fast/default.nix> { inherit pkgs; };
  vivado = import <artiq-fast/vivado.nix> { inherit pkgs; };
  zc706-fsbl = import ./fsbl.nix { inherit pkgs; };
  mkbootimage = (import ./mkbootimage.nix { inherit pkgs; });
  build-zc706 = { variant }: let
    firmware = rustPlatform.buildRustPackage rec {
      name = "zc706-${variant}-firmware";
      version = "0.1.0";

      src = ./src;
      cargoSha256 = "17qqaf3fpgxd8my721i8djkcc8y04xpi50la61s2pmr6lgdnj0wi";

      nativeBuildInputs = [
        pkgs.gnumake
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
        pkgs.cargo-xbuild
        pkgs.llvmPackages_9.llvm
        pkgs.llvmPackages_9.clang-unwrapped
      ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/src"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        make VARIANT=${variant}
      '';

      installPhase = ''
        mkdir -p $out $out/nix-support
        cp ../build/firmware/armv7-none-eabihf/release/runtime $out/runtime.elf
        cp ../build/firmware/armv7-none-eabihf/release/szl $out/szl.elf
        echo file binary-dist $out/runtime.elf >> $out/nix-support/hydra-build-products
        echo file binary-dist $out/szl.elf >> $out/nix-support/hydra-build-products
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
        ln -s ${firmware}/szl.elf $out
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
      ln -s ${firmware}/szl.elf szl.elf
      ln -s ${gateware}/top.bit top.bit
      cat > boot.bif << EOF
      the_ROM_image:
      {
        [bootloader]szl.elf
        top.bit
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
    { inherit zc706-fsbl; }
  )
