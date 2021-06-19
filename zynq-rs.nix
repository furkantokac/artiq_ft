let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "040d41fd76499f6991634ce5fb63bef879c3c694";
    sha256 = "03ym7aj6kys54vr54hff1460zrq9qqafzzr8g12wy9la0mnmr1wi";
  }
