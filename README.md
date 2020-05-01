Configure Nix channels:

```shell
nix-channel --add https://nixbld.m-labs.hk/channel/custom/artiq/fast-beta/artiq-fast
nix-channel --update
```

Pure build with Nix:

```shell
nix-build -A zc706-jtag
./remote_run.sh
```

Impure incremental build:

```shell
nix-shell
cd src
./zc706.py -g  # build gateware
make           # build firmware
cd ..
./remote_run.sh -i
```

The impure build process can also be used on non-Nix systems.
