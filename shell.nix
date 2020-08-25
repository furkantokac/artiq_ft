let
  mozillaOverlay = import (builtins.fetchTarball "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  artiq-fast = <artiq-fast>;
  rustPlatform = (import ./rustPlatform.nix { inherit pkgs; });
  artiqpkgs = import "${artiq-fast}/default.nix" { inherit pkgs; };
  vivado = import "${artiq-fast}/vivado.nix" { inherit pkgs; };
  cargo-xbuild = import ./cargo-xbuild.nix { inherit pkgs; };
in
  pkgs.stdenv.mkDerivation {
    name = "artiq-zynq-env";
    buildInputs = [
      pkgs.gnumake
      rustPlatform.rust.rustc
      rustPlatform.rust.cargo
      pkgs.llvmPackages_9.llvm
      pkgs.llvmPackages_9.clang-unwrapped
      pkgs.cacert
      cargo-xbuild

      pkgs.openocd
      pkgs.openssh pkgs.rsync

      (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
      vivado
      artiqpkgs.binutils-arm

      (import ./mkbootimage.nix { inherit pkgs; })
    ];

    XARGO_RUST_SRC = "${rustPlatform.rust.rustc.src}/src";
  }
