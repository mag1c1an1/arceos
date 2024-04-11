#!/bin/bash

# Install packages
sudo sed -i "s/http:\/\/archive.ubuntu.com/http:\/\/mirrors.tuna.tsinghua.edu.cn/g" /etc/apt/sources.list
sudo apt-get update
sudo apt-get install -y build-essential python3-mako

# Create a hypervisor image link to /lib/firmware/rvm-intel.bin
mv hv_pc-x86.bin arceos-intel.bin
sudo mkdir -p /lib/firmware
sudo ln -sf ~/arceos-intel.bin /lib/firmware

# Clone jailhouse, apply patches and build
cd jailhouse
./gen-config.sh
make

# Generate a grub config file
sudo update-grub

echo
echo "Setup OK!"
echo "Press ENTER to reboot..."
read
sudo reboot
