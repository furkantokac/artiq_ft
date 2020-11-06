let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "d623913535906739c31dc9838cdf1db4b975fb46";
    sha256 = "1bbxv1pgqkz9x5awshqh02r0ha0zc6yjabr5iwfbci8dpc63msv1";
  }
