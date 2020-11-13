let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "cb50c8d61b9ffbb4c88b5f2e96fa9f4394266c43";
    sha256 = "1wp81fm689l5ghh2bzlanqx7g066166ki39mlzhly48xczky6np2";
  }
