{ pkgs }:

let
  rustcSrc = pkgs.fetchgit {
    url = "https://github.com/rust-lang/rust.git";
    # sync with git_commit_hash from pkg.rust in channel-rust-nightly.toml
    rev = "5ef299eb9805b4c86b227b718b39084e8bf24454";
    sha256 = "0gc9hmb1sfkaf3ba8fsynl1n6bs8nk65hbhhx7ss89dfkrsxrn0x";
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
