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
  zc706 = pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zc706.git";
    rev = "526cfe7577c189687ed1fdca512120dd1460bb80";
    sha256 = "1gsnggp4237mn9pl6xsvfj859akbsw9w0vmc1i29famz1zzwly76";
  };
  rustManifest = "${zc706}/channel-rust-nightly.toml";

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
      pkgs.cargo-xbuild

      pkgs.pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc
      pkgs.openocd
      pkgs.gdb

      (pkgs.python3.withPackages(ps: (with artiqpkgs; [ migen migen-axi misoc artiq ])))
      vivado
    ];

    # Set Environment Variables
    RUST_BACKTRACE = 1;
    XARGO_RUST_SRC = "${rustcSrc}/src";
  }
