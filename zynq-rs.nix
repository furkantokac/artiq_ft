let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "ba252e72da08b40a42991bbf1950a5f4f8a3ba73";
    sha256 = "1h4zwl323y7ads2vxbf995fy7i1zkgppysbandkp3fmc8g22na62";
  }
