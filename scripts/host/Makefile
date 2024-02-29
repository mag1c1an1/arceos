QEMU ?= qemu-system-x86_64
PORT ?= 2333
WORKSPACE ?= /home/tang/server
target_bin := $(WORKSPACE)/arceos/apps/hv/hv_pc-x86.bin 
jailhouse := $(WORKSPACE)/jailhouse
# jailhouse := /home/tang/server/jailhouse
guest := $(WORKSPACE)/arceos/scripts/guest/*

qemu_image := ubuntu-20.04-server-cloudimg-amd64.img
qemu_args := \
  -smp 1 -m 4G -accel kvm -nographic \
  -machine q35,kernel_irqchip=split \
  -cpu host,-kvm-asyncpf,-kvm-pv-eoi,-kvm-pv-ipi,-kvm-pv-sched-yield,-kvm-pv-unhalt,-kvm-steal-time,-kvmclock \
  -drive file=$(qemu_image) \
  -net user,id=net,hostfwd=tcp::$(PORT)-:22 -net nic,model=e1000e \
  -serial mon:stdio

$(qemu_image):
	wget https://cloud-images.ubuntu.com/releases/focal/release/$(qemu_image)

.ONESHELL:
image: $(qemu_image)
	cat >user-data <<EOF
	#cloud-config
	password: guest
	chpasswd: { expire: False }
	ssh_pwauth: True
	EOF
	cloud-localds user-data.img user-data
	qemu-img resize $(qemu_image) +64G
	$(QEMU) $(qemu_args) -drive file=user-data.img,format=raw

qemu: $(qemu_image)
	$(QEMU) $(qemu_args)

ssh:
	ssh -p $(PORT) ubuntu@localhost

scp_bin:
	scp -P $(PORT) -r $(target_bin) ubuntu@localhost:/home/ubuntu

scp_jailhouse:
	scp -P $(PORT) -r $(jailhouse) ubuntu@localhost:/home/ubuntu

scp_guest:
	scp -P $(PORT) -r $(guest) ubuntu@localhost:/home/ubuntu
