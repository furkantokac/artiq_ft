let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "966e43e14ed26e883a2de03d56cedde964283269";
    sha256 = "15lfvc5dwx3qvkba10p0fjx2dzgsyk57ivp4izwg2dhwdcnikk6j";
  }
