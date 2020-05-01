{ pkgs }:

pkgs.stdenv.mkDerivation {
  pname = "mkbootimage";
  version = "2.2";

  src = pkgs.fetchFromGitHub {
    owner = "antmicro";
    repo = "zynq-mkbootimage";
    rev = "4ee42d782a9ba65725ed165a4916853224a8edf7";
    sha256 = "1k1mbsngqadqihzjgvwvsrkvryxy5ladpxd9yh9iqn2s7fxqwqa9";
  };

  propagatedBuildInputs = [ pkgs.libelf pkgs.pcre ];
  patchPhase =
    ''
    substituteInPlace Makefile --replace "git rev-parse --short HEAD" "echo nix"
    '';
  installPhase =
    ''
    mkdir -p $out/bin
    cp mkbootimage $out/bin
    '';
}
