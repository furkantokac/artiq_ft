let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "57d8d8fbc7087863305721bcb8fdbdd13d65bd65";
    sha256 = "0gnbaxgingrxrxi2sjd592r1630ld4ckrz4r8ajx8zp7q3bb43jj";
  }
