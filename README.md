ARTIQ on Zynq
=============

How to use
----------

#. Install ARTIQ-6 or newer.
#. Select the latest successful build on Hydra: https://nixbld.m-labs.hk/jobset/artiq/zynq
#. Search for the job named ``<board>-<variant>-sd`` (for example: ``zc706-nist_clock-sd`` or ``zc706-nist_qc2-sd``).
#. Download the ``boot.bin`` "binary distribution" and place it at the root of a FAT-formatted SD card.
#. Optionally, create a ``config.txt`` configuration file at the root of the SD card containing ``key=value`` pairs on each line. Use the ``ip``, ``ip6`` and ``mac`` keys to respectively set the IPv4, IPv6 and MAC address of the board. Configuring an IPv6 address is entirely optional. If these keys are not found, the firmware will use default values that may or may not be compatible with your network.
#. Insert the SD card in the development board and set up the board to boot from the SD card. For the ZC706, this is achieved by placing the large DIP switch SW11 in the 00110 position.
#. Power up the board. After the firmware starts successfully, it should respond to ping at its IP addresses, and boot messages can be observed from its UART at 115200bps.
#. Create and use an ARTIQ device database as usual, but set ``"target": "cortexa9"`` in the arguments of the core device.

Development instructions
------------------------

Configure Nix channels:

```shell
nix-channel --add https://nixbld.m-labs.hk/channel/custom/artiq/fast-beta/artiq-fast
nix-channel --update
```

Pure build with Nix and execution on a remote JTAG server:

```shell
nix-build -A zc706-simple-jtag  # or zc706-nist_qc2-jtag or zc706-nist_clock-jtag
./remote_run.sh
```

Impure incremental build and execution on a remote JTAG server:

```shell
nix-shell
cd src
gateware/zc706.py -g ../build/gateware  # build gateware
make                                    # build firmware
cd ..
./remote_run.sh -i
```

Notes:

#. This is known to work with Nixpkgs 20.03 and the ``nixbld.m-labs.hk`` binary substituter can also be used here (see the ARTIQ manual for the public key and instructions).
#. The impure build process is also compatible with non-Nix systems.
#. If the board is connected to the local machine, use the ``local_run.sh`` script.
