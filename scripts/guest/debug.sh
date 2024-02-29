#!/bin/bash

# Create a hypervisor image link to /lib/firmware/rvm-intel.bin
mv type1_5_helloworld_pc-x86.bin arceos-intel.bin
sudo mkdir -p /lib/firmware
sudo ln -sf ~/arceos-intel.bin /lib/firmware

