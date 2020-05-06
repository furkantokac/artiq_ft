{ pkgs ? import <nixpkgs> }:
let 

platform = "arm-none-eabi";

binutils-pkg = { stdenv, buildPackages
, fetchurl, zlib
, extraConfigureFlags ? []
}:

stdenv.mkDerivation rec {
  basename = "binutils";
  version = "2.30";
  name = "${basename}-${platform}-${version}";
  src = fetchurl {
    url = "https://ftp.gnu.org/gnu/binutils/binutils-${version}.tar.bz2";
    sha256 = "028cklfqaab24glva1ks2aqa1zxa6w6xmc8q34zs1sb7h22dxspg";
  };
  configureFlags = [
    "--enable-deterministic-archives"
    "--target=${platform}"
    "--with-cpu=cortex-a9"
    "--with-fpu=vfpv3"
    "--with-float=hard"
    "--with-mode=thumb"
  ] ++ extraConfigureFlags;
  outputs = [ "out" "info" "man" ];
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  buildInputs = [ zlib ];
  enableParallelBuilding = true;
  meta = {
    description = "Tools for manipulating binaries (linker, assembler, etc.)";
    longDescription = ''
      The GNU Binutils are a collection of binary tools.  The main
      ones are `ld' (the GNU linker) and `as' (the GNU assembler).
      They also include the BFD (Binary File Descriptor) library,
      `gprof', `nm', `strip', etc.
    '';
    homepage = http://www.gnu.org/software/binutils/;
    license = stdenv.lib.licenses.gpl3Plus;
    /* Give binutils a lower priority than gcc-wrapper to prevent a
       collision due to the ld/as wrappers/symlinks in the latter. */
    priority = "10";
  };
};


gcc-pkg = { stdenv, buildPackages
, fetchurl, gmp, mpfr, libmpc, platform-binutils
, extraConfigureFlags ? []
}:

stdenv.mkDerivation rec {
  basename = "gcc";
  version = "9.1.0";
  name = "${basename}-${platform}-${version}";
  src = fetchurl {
    url = "https://ftp.gnu.org/gnu/gcc/gcc-${version}/gcc-${version}.tar.xz";
    sha256 = "1817nc2bqdc251k0lpc51cimna7v68xjrnvqzvc50q3ax4s6i9kr";
  };
  preConfigure =
    ''
    mkdir build
    cd build
    '';
  configureScript = "../configure";
  configureFlags =
    [ "--target=${platform}"
      "--with-arch=armv7-a"
      "--with-tune=cortex-a9"
      "--with-fpu=vfpv3"
      "--with-float=hard"
      "--disable-libssp"
      "--enable-languages=c"
      "--with-as=${platform-binutils}/bin/${platform}-as"
      "--with-ld=${platform-binutils}/bin/${platform}-ld" ] ++ extraConfigureFlags;
  outputs = [ "out" "info" "man" ];
  hardeningDisable = [ "format" "pie" ];
  propagatedBuildInputs = [ gmp mpfr libmpc platform-binutils ];
  enableParallelBuilding = true;
  dontFixup = true;
};


newlib-pkg = { stdenv, fetchurl, buildPackages, platform-binutils, platform-gcc }:

stdenv.mkDerivation rec {
  pname = "newlib";
  version = "3.1.0";
  src = fetchurl {
    url = "ftp://sourceware.org/pub/newlib/newlib-${version}.tar.gz";
    sha256 = "0ahh3n079zjp7d9wynggwrnrs27440aac04340chf1p9476a2kzv";
  };

  nativeBuildInputs = [ platform-binutils platform-gcc ];

  configureFlags = [
    "--target=${platform}"

    "--with-cpu=cortex-a9"
    "--with-fpu=vfpv3"
    "--with-float=hard"
    "--with-mode=thumb"
    "--enable-interwork"
    "--disable-multilib"

    "--disable-newlib-supplied-syscalls"
    "--with-gnu-ld"
    "--with-gnu-as"
    "--disable-newlib-io-float"
    "--disable-werror"
  ];
  dontFixup = true;
};


in rec {
  binutils-bootstrap = pkgs.callPackage binutils-pkg { };
  gcc-bootstrap = pkgs.callPackage gcc-pkg {
    platform-binutils = binutils-bootstrap;
    extraConfigureFlags = [ "--disable-libgcc" ];
  };
  newlib = pkgs.callPackage newlib-pkg {
    platform-binutils = binutils-bootstrap;
    platform-gcc = gcc-bootstrap;
  };
  binutils = pkgs.callPackage binutils-pkg {
    extraConfigureFlags = [ "--with-lib-path=${newlib}/arm-none-eabi/lib" ];
  };
  gcc = pkgs.callPackage gcc-pkg {
    platform-binutils = binutils;
    extraConfigureFlags = [ "--enable-newlib" "--with-headers=${newlib}/arm-none-eabi/include" ];
  };
}
