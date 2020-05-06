{ pkgs, board ? "zc706" }:
let
  gnutoolchain = import ./gnutoolchain.nix { inherit pkgs; };
in
pkgs.stdenv.mkDerivation {
  name = "${board}-fsbl";
  src = pkgs.fetchFromGitHub {
    owner = "Xilinx";
    repo = "embeddedsw";
    rev = "65c849ed46c88c67457e1fc742744f96db968ff1";
    sha256 = "1rvl06ha40dzd6s9aa4sylmksh4xb9dqaxq462lffv1fdk342pda";
  };
  nativeBuildInputs = [
    pkgs.gnumake
    gnutoolchain.binutils
    gnutoolchain.gcc
  ];
  patchPhase =
    ''
    patchShebangs lib/sw_apps/zynq_fsbl/misc/copy_bsp.sh
    echo 'SEARCH_DIR("${gnutoolchain.newlib}/arm-none-eabi/lib");' >> lib/sw_apps/zynq_fsbl/src/lscript.ld
    '';
  buildPhase =
    ''
    cd lib/sw_apps/zynq_fsbl/src
    make BOARD=${board}
    '';
  installPhase = 
    ''
    mkdir $out
    cp fsbl.elf $out
    '';
  doCheck = false;
  dontFixup = true;
}
