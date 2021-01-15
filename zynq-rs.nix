let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "4aa252546f3559c9060ce731943372ad30d16eef";
    sha256 = "0q9b75w6mbnsyyryainng27glxwhis325bmpv4bvzm7r9almsvks";
  }
