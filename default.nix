{
  mozillaOverlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
}:

let
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  artiq-fast = <artiq-fast>;
  rustPlatform = (import ./rustPlatform.nix { inherit pkgs; });
  buildFirmware = { name, src }:
    rustPlatform.buildRustPackage rec {
      inherit name;
      version = "0.1.0";

      inherit src;
      cargoSha256 = (import "${src}/cargosha256.nix");

      nativeBuildInputs = [ pkgs.cargo-xbuild pkgs.llvm_9 pkgs.clang_9 ];
      buildPhase = ''
        export XARGO_RUST_SRC="${rustPlatform.rust.rustc.src}/src"
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        cargo xbuild --release -p ${name}
      '';

      doCheck = false;
      installPhase = ''
        mkdir -p $out $out/nix-support
        cp target/armv7-none-eabihf/release/${name} $out/${name}.elf
        echo file binary-dist $out/${name}.elf >> $out/nix-support/hydra-build-products
      '';
      dontFixup = true;
    };

    artiqpkgs = import "${artiq-fast}/default.nix" { inherit pkgs; };
    vivado = import "${artiq-fast}/vivado.nix" { inherit pkgs; };
in
  rec {
    zc706-runtime-src = pkgs.runCommand "zc706-runtime-src"
      { buildInputs = [ 
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
      ]; }
      ''
        cp --no-preserve=mode,ownership -R ${./firmware} $out
        cd $out/runtime/src
        python ${./zc706.py} rustif
      '';
    zc706-runtime = buildFirmware { name = "runtime"; src = zc706-runtime-src; };
    zc706-szl-src = pkgs.runCommand "zc706-szl-src"
      { nativeBuildInputs = [ pkgs.llvm_9 ]; }
      ''
        cp --no-preserve=mode,ownership -R ${./firmware} $out
        llvm-objcopy -O binary ${zc706-runtime}/runtime.elf $out/szl/src/payload.bin
        lzma $out/szl/src/payload.bin
      '';
    zc706-szl = buildFirmware { name = "szl"; src = zc706-szl-src; };
    zc706-gateware = pkgs.runCommand "zc706-gateware"
      { buildInputs = [ 
        (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
        vivado
      ]; }
      ''
        python ${./zc706.py} gateware
        mkdir -p $out $out/nix-support
        cp build/top.bit $out
        echo file binary-dist $out/top.bit >> $out/nix-support/hydra-build-products
      '';
    zc706-jtag = pkgs.runCommand "zc706-jtag" {}
      ''
        mkdir $out
        ln -s ${zc706-szl}/szl $out
        ln -s ${zc706-gateware}/top.bit $out
      '';
  }
