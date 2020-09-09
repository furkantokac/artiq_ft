let
  pkgs = import <nixpkgs> {};
in
pkgs.fetchgit {
  url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
  rev = "450ccef18e609aa6c88540d6791e0b41d2de01d2";
  sha256 = "0xlvxczfwyk5zij4gnbxjvcq3hmhjslmfswp6vzl67ddkpc8bb6s";
}
