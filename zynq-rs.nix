let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "8432ff3e30a792b723e81d7bd130637dadc23268";
    sha256 = "0a729n7cj4ms752dby61ablm7200abcmb5cb5imda5kgk666msjm";
  }
