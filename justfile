app_path := "apps/task/" + app_name
app_name := "parallel"

hv-smp := "3"
guest-smp := "3"

hv-log := "debug"
guest-log := "debug"

mode := "debug"

HV_ELF := "apps/hv/hv_pc-x86-hv.elf"
HV_BIN := "apps/hv/hv_pc-x86-hv.bin"
GUEST_ELF := app_path + "/" + app_name + "_pc-x86.elf"
GUEST_BIN := app_path + "/" + app_name + "_pc-x86.bin"

clean:
  make A={{app_path}} clean
  make A=apps/hv clean

guest:
  make ARCH=x86_64 A={{app_path}} PLATFORM=pc-x86 LOG={{guest-log}} MODE={{mode}} SMP={{guest-smp}} build

guest-run:
  make ARCH=x86_64 A={{app_path}} PLATFORM=pc-x86 LOG={{guest-log}} MODE={{mode}} SMP={{guest-smp}} run

build:
  make ARCH=x86_64 A=apps/hv HV=y PLATFORM=pc-x86-hv LOG={{hv-log}} GUEST=nimbos MODE={{mode}} SMP={{hv-smp}} build

run *flags: guest build
  qemu-system-x86_64 -m 3G -smp {{hv-smp}} -machine q35 -kernel apps/hv/hv_pc-x86-hv.elf -device loader,addr=0x4000000,file=apps/hv/guest/nimbos/rvm-bios.bin,force-raw=on -device loader,addr=0x4001000,file={{GUEST_BIN}},force-raw=on -nographic -no-reboot -cpu host -accel kvm {{flags}}

gdb:
	gdb-multiarch {{HV_ELF}} \
	  -ex 'target remote localhost:1234'
