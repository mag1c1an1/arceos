OUT ?= out

entry-src := entry.S
entry-obj := $(OUT)/entry.o
loader-src := loader.c
loader-obj := $(OUT)/loader.o
virt-int-src := virt_int.S
virt-int-obj := $(OUT)/virt_int.o
virt-int-c-src := virt_int.c
virt-int-c-obj := $(OUT)/virt_int_c.o

ldscript := vlbl.lds
target := $(OUT)/vlbl
target-objs := $(entry-obj) $(loader-obj) $(virt-int-obj) $(virt-int-c-obj)
target-elf := $(target).elf
target-bin := $(target).bin
target-disasm-16 := $(target).16.asm
target-disasm-32 := $(target).32.asm

CC ?= gcc
AS ?= as
LD ?= ld
OBJCOPY ?= objcopy
OBJDUMP ?= objdump

all: $(OUT) $(target).bin disasm

disasm: disasm16 disasm32

disasm16:
	$(OBJDUMP) -D -m i8086 -M intel $(target-elf) > $(target-disasm-16)

disasm32:
	$(OBJDUMP) -D -m i386 -M intel $(target-elf) > $(target-disasm-32)

$(OUT):
	mkdir -p $(OUT)

$(entry-obj): $(entry-src)
	$(AS) --32 -msyntax=intel -mnaked-reg $< -o $@

$(loader-obj): $(loader-src)
	$(CC) -m32 -fno-pie -fno-builtin -Os -c $< -o $@

$(virt-int-obj): $(virt-int-src)
	$(AS) --32 -msyntax=intel -mnaked-reg $< -o $@

$(virt-int-c-obj): $(virt-int-c-src)
	$(CC) -m16 -march=i386 -fno-pie -fno-builtin -Os -c $< -o $@

$(target-elf): $(target-objs) $(ldscript)
	$(LD) -T$(ldscript) $(target-objs) -o $@

$(target-bin): $(target-elf)
	$(OBJCOPY) $< --strip-all -O binary $@

clean:
	rm -rf $(OUT)

.PHONY: all disasm clean
