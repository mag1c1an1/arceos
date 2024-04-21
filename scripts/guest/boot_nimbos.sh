JH_DIR=~/jailhouse-arceos
JH=$JH_DIR/tools/jailhouse

echo "create axvm nimbos"
sudo $JH axvm create 2 1 ./rvm-bios.bin ./nimbos-x86.bin