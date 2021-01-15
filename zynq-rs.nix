let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "b4d91e7904423b6477eb1f25b8cc3940e197e9c2";
    sha256 = "0g77s9jmbyxzkpnn2rs2sya5ia2admgkn74kzl2h1n4nckfk2nn6";
  }
