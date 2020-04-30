{ pkgs }:

let
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
in
  pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
    rustc = rust // { src = rustcSrc; };
    cargo = rust;
  })
