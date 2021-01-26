let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "6e6612bc3e12e50b4f6e61cde47100c3d4ab982a";
    sha256 = "14hj77n9g2zack6sjgs7337j8yq9r3jrpdsmc62kmxfbbmy8jhlg";
  }
