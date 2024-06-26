app_path := "apps/" + app_name
app_name := "helloworld"

hv-smp := "2"
guest-smp := "2"

log := "debug"
mode := "debug"

HV_ELF := "apps/hv/hv_pc-x86-hv.elf"
HV_BIN := "apps/hv/hv_pc-x86-hv.bin"
GUEST_ELF := app_path + "/" + app_name + "_pc-x86.elf"
GUEST_BIN := app_path + "/" + app_name + "_pc-x86.bin"

clean:
  make A={{app_path}} clean
  make A=apps/hv clean

guest:
  make ARCH=x86_64 A={{app_path}} PLATFORM=pc-x86 LOG={{log}} MODE={{mode}} SMP={{guest-smp}} build

guest-run:
  make ARCH=x86_64 A={{app_path}} PLATFORM=pc-x86 LOG={{log}} MODE={{mode}} SMP={{guest-smp}} run

build:
  make ARCH=x86_64 A=apps/hv HV=y PLATFORM=pc-x86-hv LOG={{log}} GUEST=nimbos MODE={{mode}} SMP={{hv-smp}} build

hv-build:
  cargo rustc --target x86_64-unknown-none --target-dir target --features "libax/platform-pc-x86 libax/log-level-info libax/hv  libax/irq libax/bus-pci "  --manifest-path apps/hv/Cargo.toml -- -Clink-args="-T/home/maji/hv-related/arceos/modules/axhal/linker_pc-x86_hv.lds -no-pie"
  cp target/x86_64-unknown-none/release/arceos-hv apps/hv/hv_pc-x86.elf
  rust-objcopy --binary-architecture=x86_64 apps/hv/hv_pc-x86.elf --strip-all -O binary apps/hv/hv_pc-x86.bin

run *flags: build guest
  qemu-system-x86_64 -m 3G -smp {{hv-smp}} -machine q35 -kernel apps/hv/hv_pc-x86-hv.elf -device loader,addr=0x4000000,file=apps/hv/guest/nimbos/rvm-bios.bin,force-raw=on -device loader,addr=0x4001000,file={{GUEST_BIN}},force-raw=on -nographic -no-reboot -cpu host -accel kvm {{flags}}

gdb:
	gdb-multiarch {{HV_ELF}} \
	  -ex 'target remote localhost:1234'
