let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "7360984efbd772ae992ef00af09786b0ae8430f0";
    sha256 = "10xrkhvrs6p0pn50cccvbnzi7l9lp8a6xmqy0pv5vg0f1qq3zxif";
  }
