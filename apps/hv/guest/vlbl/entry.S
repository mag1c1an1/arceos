# it's 7c00 here
.section .text
.code16
.global entry
entry:
    cli
    cld

    # 初始化段寄存器
    xor     ax, ax
    mov     ds, ax
    mov     es, ax
    mov     ss, ax

    # 进入保护模式
    lgdt    [prot_gdt_desc]
    mov     eax, cr0
    or      eax, 0x1
    mov     cr0, eax

    ljmp    0x8, entry32

.code32
.global entry32
entry32:
    # 初始化段选择子到数据段
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    # 初始化临时栈
    mov     ebp, 0x7b00
    mov     esp, 0x7b00

    # 加载内核
    push    0x00199bef # initramfs 大小
    push    0x72000000 # initramfs 地址
    push    0x00100000 # 32-bit 加载地址
    push    0x0001e000 # 栈顶地址
    push    0x00010000 # 16-bit 加载地址
    push    0x70200000 # 镜像地址
    call    load_kernel

    # 跳转到16-bit保护模式
    lgdt    [prot_16_gdt_desc]
    ljmp    0x8, entry_prot_16


.code16
.global entry_prot_16
entry_prot_16:
    # 再次初始化段选择子到数据段
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    # 加载中断向量表
    lidt    [real_idt_dest]

    # 关闭保护模式
    mov     eax, cr0
    or      eax, 0x1
    xor     eax, 0x1
    mov     cr0, eax

    # 跳转到实模式
    ljmp    0x0, entry_last_jump

.code16
.global entry_last_jump
entry_last_jump:
    # 初始化段寄存器和栈，准备启动
    mov     ax, 0x1000
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax
    mov     ss, ax
    mov     sp, 0xe000

    # 清理寄存器
    xor     eax, eax
    xor     ebx, ebx
    xor     ecx, ecx
    xor     edx, edx
    xor     ebp, ebp
    xor     esi, esi
    xor     edi, edi

    # 跳转到linux内核入口
    ljmp    0x1020, 0

    # 以下代码不应该被执行，使用一条无效的in指令返回
    in      ax, 0x11
    jmp $

.global end_of_code
end_of_code:

# 32位GDT
.balign 16
.global prot_gdt
prot_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x10: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)

.global prot_gdt_desc
prot_gdt_desc:
    .short  prot_gdt_desc - prot_gdt - 1    # limit
    .long   prot_gdt                        # base

# 16位GDT
.balign 16
.global prot_16_gdt
prot_16_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x000f9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=16bit code exec/read, DPL=0, byte)
    .quad 0x000f93000000ffff    # 0x10: data segment (base=0, limit=0xfffff, type=16bit data read/write, DPL=0, byte)

.global prot_16_gdt_desc
prot_16_gdt_desc:
    .short  prot_16_gdt_desc - prot_16_gdt - 1    # limit
    .long   prot_16_gdt                        # base
