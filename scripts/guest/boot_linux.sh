JH_DIR=~/jailhouse-arceos
JH=$JH_DIR/tools/jailhouse

echo "create axvm linux"
sudo $JH axvm create 2 2 ./vlbl.bin ./bzImage.bin ./initramfs.cpio.gz