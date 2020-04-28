let
  mozillaOverlay = import (builtins.fetchTarball "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  artiq-fast = builtins.fetchTarball "https://nixbld.m-labs.hk/channel/custom/artiq/fast-beta/artiq-fast/nixexprs.tar.xz";

  pkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };

  rustcSrc = pkgs.fetchgit {
    url = "https://github.com/rust-lang/rust.git";
    # master of 2020-04-10
    rev = "94d346360da50f159e0dc777dc9bc3c5b6b51a00";
    sha256 = "1hcqdz4w2vqb12rrqqcjbfs5s0w4qwjn7z45d1zh0fzncdcf6f7d";
    fetchSubmodules = true;
  };
  rustManifest = ./channel-rust-nightly.toml;

  targets = [];
  rustChannelOfTargets = _channel: _date: targets:
    (pkgs.lib.rustLib.fromManifestFile rustManifest {
      inherit (pkgs) stdenv fetchurl patchelf;
    }).rust.override { inherit targets; };
  rust =
    rustChannelOfTargets "nightly" null targets;
  rustPlatform = pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  });

  artiqpkgs = import "${artiq-fast}/default.nix" { inherit pkgs; };
  vivado = import "${artiq-fast}/vivado.nix" { inherit pkgs; };
in
  pkgs.stdenv.mkDerivation {
    name = "artiq-zynq-env";
    buildInputs = [
      rustPlatform.rust.rustc
      rustPlatform.rust.cargo
      rustcSrc
      pkgs.clang_9
      pkgs.cacert
      pkgs.cargo-xbuild

      pkgs.openssh pkgs.rsync

      (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
      vivado
      pkgs.llvm_9
      pkgs.lld_9
    ];

    XARGO_RUST_SRC = "${rustcSrc}/src";
  }
