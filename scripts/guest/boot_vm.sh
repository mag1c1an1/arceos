JH_DIR=~/jailhouse-arceos
JH=$JH_DIR/tools/jailhouse

echo "create axvm"
sudo $JH axvm create 2 ./rvm-bios.bin ./nimbos-x86.bin