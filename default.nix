{
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rustPlatform = (import ./rustPlatform.nix { inherit pkgs; });
  artiqpkgs = import <artiq-fast/default.nix> { inherit pkgs; };
  vivado = import <artiq-fast/vivado.nix> { inherit pkgs; };
  mkbootimage = (import ./mkbootimage.nix { inherit pkgs; });
in
  rec {
    zc706-firmware = rustPlatform.buildRustPackage rec {
      name = "zc706-firmware";
      version = "0.1.0";

      src = ./src;
      cargoSha256 = "1b40w3ycc0hx6hahxgz935vv01q1lirbrn4cb4k0r3dmgzvsdk6l";

      nativeBuildInputs = [
        pkgs.gnumake
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
        pkgs.cargo-xbuild
        pkgs.llvm_9
        pkgs.clang_9
      ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/src"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        make clean
        make
      '';

      installPhase = ''
        mkdir -p $out $out/nix-support
        cp target/armv7-none-eabihf/release/runtime $out/runtime.elf
        cp target/armv7-none-eabihf/release/szl $out/szl.elf
        echo file binary-dist $out/runtime.elf >> $out/nix-support/hydra-build-products
        echo file binary-dist $out/szl.elf >> $out/nix-support/hydra-build-products
      '';

      doCheck = false;
      dontFixup = true;
    };
    zc706-gateware = pkgs.runCommand "zc706-gateware"
      {
        nativeBuildInputs = [ 
          (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
          vivado
        ];
      }
      ''
        python ${./src/zc706.py} -g
        mkdir -p $out $out/nix-support
        cp build/top.bit $out
        echo file binary-dist $out/top.bit >> $out/nix-support/hydra-build-products
      '';

    # SZL startup
    zc706-jtag = pkgs.runCommand "zc706-jtag" {}
      ''
        mkdir $out
        ln -s ${zc706-firmware}/szl.elf $out
        ln -s ${zc706-gateware}/top.bit $out
      '';
    zc706-sd = pkgs.runCommand "zc706-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      bif=`mktemp`
      cat > $bif << EOF
      the_ROM_image:
      {
        [bootloader]${zc706-firmware}/szl.elf
      }
      EOF
      mkdir $out
      mkbootimage $bif $out/boot.bin
      ln -s ${zc706-gateware}/top.bit $out
      '';
    zc706-sd-zip = pkgs.runCommand "zc706-sd-zip"
      {
        buildInputs = [ pkgs.zip ];
      }
      ''
        mkdir -p $out $out/nix-support
        zip -j $out/sd.zip ${zc706-sd}/*
        echo file binary-dist $out/sd.zip >> $out/nix-support/hydra-build-products
      '';

    # FSBL startup
    zc706-fsbl = import ./fsbl.nix { inherit pkgs; };
    zc706-fsbl-sd = pkgs.runCommand "zc706-fsbl-sd"
      {
        buildInputs = [ mkbootimage ];
      }
      ''
      bif=`mktemp`
      cat > $bif << EOF
      the_ROM_image:
      {
        [bootloader]${zc706-fsbl}/fsbl.elf
        ${zc706-gateware}/top.bit
        ${zc706-firmware}/runtime.elf
      }
      EOF
      mkdir $out $out/nix-support
      mkbootimage $bif $out/boot.bin
      echo file binary-dist $out/boot.bin >> $out/nix-support/hydra-build-products
      '';
  }
