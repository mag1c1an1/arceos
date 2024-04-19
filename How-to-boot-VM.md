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

You can log in to the Linux CLI using the SSH script prepared in `scripts/host`, this way, you can access the Linux environment using the SSH port instead of the QEMU serial port, as the QEMU serial port will be occupied by ArceOS-HV and guest VMs.

* On host

```bash
# Execute in the scripts/host directory. 
make ssh
```

* Inside Linux guest

```bash
# Execute in guest /home/ubuntu directory.
./setup.sh

./enable-arceos-hv.sh
```

You can see that arceos-hypervisor is booted and initialized, and then it returns to the Linux environment. 

At this point, Linux has been downgraded to a guest VM running on the arceos-hypervisor.

Then you can start another guest VM through jailhouse cmd tool.

```bash
# Execute in guest /home/ubuntu directory.
sudo ${PATH_TO_JAILHOUSE_TOOL} axvm create CPU_MASK BIOS_IMG KERNEL_IMG
```

There is also a script for it.

```bash
# Execute in guest /home/ubuntu directory.
./boot_vm.sh
```

Currently only [Nimbos](https://github.com/equation314/nimbos) is well supported, you can find its bios [here](apps/hv/guest/nimbos/bios).