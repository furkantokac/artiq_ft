let
  zynq-rs = (import ./zynq-rs.nix);
  pkgs = import <nixpkgs> { overlays = [ (import "${zynq-rs}/nix/mozilla-overlay.nix") ]; };
  rustPlatform = (import "${zynq-rs}/nix/rust-platform.nix" { inherit pkgs; });
  cargo-xbuild = (import zynq-rs).cargo-xbuild;
  mkbootimage = import "${zynq-rs}/nix/mkbootimage.nix" { inherit pkgs; };
  artiqpkgs = import <artiq-fast/default.nix> { inherit pkgs; };
  vivado = import <artiq-fast/vivado.nix> { inherit pkgs; };
  # FSBL configuration supplied by Vivado 2020.1 for these boards:
  fsblTargets = ["zc702" "zc706" "zed"];
  sat_variants = ["satellite" "nist_clock_satellite" "nist_qc2_satellite" "acpki_nist_clock_satellite" "acpki_nist_qc2_satellite"];
  build = { target, variant, json ? null }: let
    szl = (import zynq-rs)."${target}-szl";
    fsbl = import "${zynq-rs}/nix/fsbl.nix" {
      inherit pkgs;
      board = target;
    };
    fwtype = if builtins.elem variant sat_variants then "satman" else "runtime";

    firmware = rustPlatform.buildRustPackage rec {
      # note: due to fetchCargoTarball, cargoSha256 depends on package name
      name = "firmware";

      src = ./src;
      cargoSha256 = "sha256-uiwESZNwPdVnDkA1n0v1DQHp3rTazDkgIYscVTpgNq0=";

      nativeBuildInputs = [
        pkgs.gnumake
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ ps.jsonschema migen migen-axi misoc artiq ])))
        cargo-xbuild
        pkgs.llvmPackages_9.llvm
        pkgs.llvmPackages_9.clang-unwrapped
      ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc}/lib/rustlib/src/rust/library"
        export CLANG_EXTRA_INCLUDE_DIR="${pkgs.llvmPackages_9.clang-unwrapped.lib}/lib/clang/9.0.1/include"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        make TARGET=${target} GWARGS="${if json == null then "-V ${variant}" else json}" ${fwtype}
      '';

      installPhase = ''
        mkdir -p $out $out/nix-support
        cp ../build/${fwtype}.bin $out/${fwtype}.bin
        cp ../build/firmware/armv7-none-eabihf/release/${fwtype} $out/${fwtype}.elf
        echo file binary-dist $out/${fwtype}.bin >> $out/nix-support/hydra-build-products
        echo file binary-dist $out/${fwtype}.elf >> $out/nix-support/hydra-build-products
      '';

      doCheck = false;
      dontFixup = true;
    };
    gateware = pkgs.runCommand "${target}-${variant}-gateware"
      {
        nativeBuildInputs = [ 
          (pkgs.python3.withPackages(ps: (with artiqpkgs; [ ps.jsonschema migen migen-axi misoc artiq ])))
          vivado
        ];
      }
      ''
        python ${./src/gateware}/${target}.py -g build ${if json == null then "-V ${variant}" else json}
        mkdir -p $out $out/nix-support
        cp build/top.bit $out
        echo file binary-dist $out/top.bit >> $out/nix-support/hydra-build-products
      '';

    # SZL startup
    jtag = pkgs.runCommand "${target}-${variant}-jtag" {}
      ''
        mkdir $out
        ln -s ${szl}/szl.elf $out
        ln -s ${firmware}/${fwtype}.bin $out
        ln -s ${gateware}/top.bit $out
      '';
    sd = pkgs.runCommand "${target}-${variant}-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      # Do not use "long" paths in boot.bif, because embedded developers
      # can't write software (mkbootimage will segfault).
      bifdir=`mktemp -d`
      cd $bifdir
      ln -s ${szl}/szl.elf szl.elf
      ln -s ${firmware}/${fwtype}.elf ${fwtype}.elf
      ln -s ${gateware}/top.bit top.bit
      cat > boot.bif << EOF
      the_ROM_image:
      {
        [bootloader]szl.elf
        top.bit
        ${fwtype}.elf
      }
      EOF
      mkdir $out $out/nix-support
      mkbootimage boot.bif $out/boot.bin
      echo file binary-dist $out/boot.bin >> $out/nix-support/hydra-build-products
      '';

    # FSBL startup
    fsbl-sd = pkgs.runCommand "${target}-${variant}-fsbl-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      bifdir=`mktemp -d`
      cd $bifdir
      ln -s ${fsbl}/fsbl.elf fsbl.elf
      ln -s ${gateware}/top.bit top.bit
      ln -s ${firmware}/${fwtype}.elf ${fwtype}.elf
      cat > boot.bif << EOF
      the_ROM_image:
      {
        [bootloader]fsbl.elf
        top.bit
        ${fwtype}.elf
      }
      EOF
      mkdir $out $out/nix-support
      mkbootimage boot.bif $out/boot.bin
      echo file binary-dist $out/boot.bin >> $out/nix-support/hydra-build-products
      '';
  in {
    "${target}-${variant}-firmware" = firmware;
    "${target}-${variant}-gateware" = gateware;
    "${target}-${variant}-jtag" = jtag;
    "${target}-${variant}-sd" = sd;
  } // (
    if builtins.elem target fsblTargets
    then {
      "${target}-${variant}-fsbl-sd" = fsbl-sd;
    }
    else {}
  );
in
  (
    (build { target = "zc706"; variant = "nist_clock"; }) //
    (build { target = "zc706"; variant = "nist_clock_master"; }) //
    (build { target = "zc706"; variant = "nist_clock_satellite"; }) //
    (build { target = "zc706"; variant = "nist_qc2"; }) //
    (build { target = "zc706"; variant = "nist_qc2_master"; }) //
    (build { target = "zc706"; variant = "nist_qc2_satellite"; }) //
    (build { target = "zc706"; variant = "acpki_nist_clock"; }) //
    (build { target = "zc706"; variant = "acpki_nist_clock_master"; }) //
    (build { target = "zc706"; variant = "acpki_nist_clock_satellite"; }) //
    (build { target = "zc706"; variant = "acpki_nist_qc2"; }) //
    (build { target = "zc706"; variant = "acpki_nist_qc2_master"; }) //
    (build { target = "zc706"; variant = "acpki_nist_qc2_satellite"; }) //
    (build { target = "kasli_soc"; variant = "demo"; json = ./demo.json; }) //
    (build { target = "kasli_soc"; variant = "master"; json = ./kasli-soc-master.json; }) //
    (build { target = "kasli_soc"; variant = "satellite"; json = ./kasli-soc-satellite.json; }) //
    { inherit zynq-rs; }
  )
