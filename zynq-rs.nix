let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "f83ef218de3420192d9096d7c1c9ce3794858b8a";
    sha256 = "1hg89akb9i9my9n9v5vhdw8m4zq3gq65ali1lmf45igq0a3dr5ma";
  }
