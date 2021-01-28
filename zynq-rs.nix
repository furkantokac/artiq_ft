let
  pkgs = import <nixpkgs> {};
in
  pkgs.fetchgit {
    url = "https://git.m-labs.hk/M-Labs/zynq-rs.git";
    rev = "78d58d17ec7906a6cadd1678576939d20612cf8f";
    sha256 = "1y74i7j9kawhlq22zyicjsxldx9f7h4i22yabw1z4qga19zv6qjd";
  }
