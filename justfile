run:
  make ARCH=x86_64 A=apps/hv HV=y LOG=info GUEST=nimbos run

hv-build:
  cargo rustc --target x86_64-unknown-none --target-dir /home/maji/hv-related/hv/arceos/target --release  --features "libax/platform-pc-x86 libax/log-level-trace libax/hv  libax/irq libax/bus-pci "  --manifest-path apps/hv/Cargo.toml -- -Clink-args="-T/home/maji/hv-related/hv/arceos/modules/axhal/linker_pc-x86_hv.lds -no-pie"
  cp /home/maji/hv-related/hv/arceos/target/x86_64-unknown-none/release/arceos-hv apps/hv/hv_pc-x86.elf
  rust-objcopy --binary-architecture=x86_64 apps/hv/hv_pc-x86.elf --strip-all -O binary apps/hv/hv_pc-x86.bin

hw:
  qemu-system-x86_64 -m 3G -smp 1 -machine q35 -kernel apps/hv/hv_pc-x86.elf -device loader,addr=0x4000000,file=apps/hv/guest/nimbos/rvm-bios.bin,force-raw=on -device loader,addr=0x4001000,file=apps/helloworld/helloworld_pc-x86.bin,force-raw=on -nographic -cpu host -accel kvm

mc: 
  qemu-system-x86_64 -m 3G -smp 2 -machine q35 -kernel apps/hv/hv_pc-x86.elf -device loader,addr=0x4000000,file=apps/hv/guest/nimbos/rvm-bios.bin,force-raw=on -device loader,addr=0x4001000,file=apps/task/parallel/parallel_pc-x86.bin,force-raw=on -nographic -cpu host -accel kvm

parp:
  make ARCH=x86_64 A=apps/task/parallel HV=n LOG=info  SMP=4 run
