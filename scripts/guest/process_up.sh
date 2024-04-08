JH_DIR=~/jailhouse
JH=$JH_DIR/tools/jailhouse

echo "create axtask"
sudo $JH axtask up 2 1 ./rvm-bios.bin
