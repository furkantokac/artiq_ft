let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "6266d280951c3fe5d4963a4b1ca45ce369d6b773";
    sha256 = "13cnxlnafj5s0rd9a9k369b6xfb1gnvp40vfk81kbr1924a1c04d";
  }
