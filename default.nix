{
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  artiq-fast = <artiq-fast>;
  rustPlatform = (import ./rustPlatform.nix { inherit pkgs; });
  artiqpkgs = import "${artiq-fast}/default.nix" { inherit pkgs; };
  vivado = import "${artiq-fast}/vivado.nix" { inherit pkgs; };
in
  rec {
    zc706-szl = rustPlatform.buildRustPackage rec {
      name = "szl";
      version = "0.1.0";

      src = ./src;
      cargoSha256 = "199qfs7fbbj8kxkyb0dcns6hdq9hvlppk7l3pnz204j9zkd7dkcp";

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
        cp target/armv7-none-eabihf/release/szl $out/szl.elf
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
    zc706-jtag = pkgs.runCommand "zc706-jtag" {}
      ''
        mkdir $out
        ln -s ${zc706-szl}/szl.elf $out
        ln -s ${zc706-gateware}/top.bit $out
      '';
  }
