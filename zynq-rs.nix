let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "c7e9f85de2b204d6889c1c44b8ef4e587d0eb6a9";
    sha256 = "1zxbdi01cbxcbfb3in3bcyhrwwsnyqgb8nzwzsqnn8sw16cjkfxv";
  }
