# ArceOS-Hypervisor

This document contains some useful commands for running the ArceOS hypervisor as well as boot guest VM.

To boot ArceOS hypervisor from Linux, it relies on our modified jailhouse kernel module driver. 

When using the following script, make sure that the [jailhouse-arceos](https://github.com/arceos-hypervisor/jailhouse-arceos) repository is located at the same level as this arceos repository.

Otherwise, you may need to find a way to copy the jailhouse-arceos folder into the Linux filesystem using your own method.

## Setup environment 

Boot Linux upon QEMU.

There are some scripts in `scripts/host` which may help you to setup a Linux rootfs.

Firstly, prepare a Linux environment through [cloud-init](https://cloud-init.io/) and boot it upon QEMU.

```bash
# Enter scripts/host directory. 
cd scripts/host
make image
```
You only need to run upon commands once for downloading and configuration.

Execute this command only for subsequent runs.
```bash
# Execute in the scripts/host directory. 
make qemu
```

Note that when starting QEMU, you need to **open another terminal and use telnet to connect to the corresponding port**. The QEMU we use provides two serial ports for the upper-level virtual machine: COM0 at 0x3f8 and COM1 at 0x2f8. COM0 is connected to mon:std, while COM1 is bound to the lo loopback interface, here TCP port 4321.

```bash
telnet localhost 4321
```

See this [script](scripts/host/Makefile) for details.

## Compile ArceOS-HV

Then, compile the ArceOS-HV itself in its root directory.

```bash
make A=apps/hv HV=y TYPE1_5=y ARCH=x86_64 STRUCT=Hypervisor GUEST=nimbos LOG=debug SMP=2 build
# You can also use this command which will copy the binary image file into Linux rootfs automatically.
make A=apps/hv HV=y TYPE1_5=y ARCH=x86_64 STRUCT=Hypervisor GUEST=nimbos LOG=debug SMP=2 scp_linux
```

## Copy scripts and image files

The files inside the `scripts/guest` need to be copied to the Linux rootfs.

We have prepared ready-to-use copy scripts in `scripts/host`. 

```bash
# Execute in the scripts/host directory. 
./scp.sh
```
For specific information, please refer to `scripts/host/Makefile`.

## Setup environment inside Linux rootfs.

**The remaining steps need to be performed within the Linux environment that we just booted.**

You can log in to the Linux CLI using the SSH script prepared in `scripts/host`, this way, you can access the Linux environment using the SSH port instead of the QEMU serial port (COM1, bind to local loopback interface as we mensioned before).

* On host

```bash
# Execute in the scripts/host directory. 
make ssh
```

* Inside Linux guest

```bash
# Execute in guest /home/ubuntu directory.
./setup.sh
```

After this step, the console of the host Linux is modified to ttyS1, which corresponds to COM1 at 0x2f8.

## Boot arceos-hypervisor

Before starting arceos-hypervisor, pay attention to the `gen-config.sh` file in the [jailhouse-arceos](https://github.com/arceos-hypervisor/jailhouse-arceos) folder. 

Here, you need to set the reserved memory space size for arceos-hypervisor based on the hardware memory information.

Current script defaults to reserving 4GB of memory for arceos-hypervisor, just like this:
```bash
# Line 2
sudo python3 ./tools/jailhouse-config-create --mem-hv 4G ./configs/x86/qemu-arceos.c
# ...
# Line 13
cmdline='memmap=0x100000000\\\\\\$0x100000000 console=ttyS1'
```
**If your hardware doesn't have that much memory, remember to reduce this memory size!!!**.

The size of the reserved memory space needs to be larger than the physical memory size specified in the arceos [configuration file](modules/axconfig/src/platform/pc-x86-hv-type15.toml).


We have prepared a script to boot the arceos-hypervisor. Run this command in user space on the host Linux.

```bash
# Execute in guest /home/ubuntu directory.
./enable-arceos-hv.sh
```

You can see that arceos-hypervisor is booted and initialized, and then it returns to the Linux environment. 

At this point, Linux has been downgraded to a guest VM running on the arceos-hypervisor.

Then you can start another guest VM through jailhouse cmd tool.

```bash
# Execute in guest /home/ubuntu directory.
sudo ${PATH_TO_JAILHOUSE_TOOL} axvm create CPU_MASK VM_TYPE BIOS_IMG KERNEL_IMG RAMDISK_IMG
```

There is also some scripts for it.

```bash
# Execute in guest /home/ubuntu directory.
./boot_nimbios.sh
./boot_linux.sh
```

## Boot Guest VM

### [NimbOS](https://github.com/equation314/nimbos)
You can find its bios [here](apps/hv/guest/nimbos/bios).

### Linux
Currently, the vanilla Linux kernel is not supported (though I hope it will be).

This modified Linux kernel [linux-5.10.35](https://github.com/arceos-hypervisor/linux-5.10.35-rt/tree/tracing) with RT patch can run on arceos-hypervisor.

You need [vlbl](apps/hv/guest/vlbl) for bootloader, you can find vlbl.bin in its target dir.
You need to build your own ramdisk image, you can find helpful guides [here](https://github.com/OS-F-4/usr-intr/blob/main/ppt/%E5%B1%95%E7%A4%BA%E6%96%87%E6%A1%A3/linux-kernel.md#%E5%88%9B%E5%BB%BA%E6%96%87%E4%BB%B6%E7%B3%BB%E7%BB%9F%E4%BB%A5busybox%E4%B8%BA%E4%BE%8B).

## Emulated Devices

We are working on virtio devices...