.equ NUM_INT, 256

.altmacro
.macro INT_HANDLER, i
    .global .int_handler_\i
    .int_handler_\i:
    pushd   \i
    jmp     int_handler_common
.endm

.macro IDT_ENTRY, i
    .short .int_handler_\i
    .short 0
.endm

# 虚拟IDT
.section .text
.balign 16
.global virt_idt
virt_idt:
.set i, 0
.rept NUM_INT
    IDT_ENTRY %i
    .set i, i + 1
.endr

.global real_idt_dest
real_idt_dest:
    .short 0x3ff # limit
    .long virt_idt # base

.code16
.balign 16
.global .int_handlers
.int_handlers:
.set i, 0
.rept NUM_INT
    INT_HANDLER %i
    .set i, i + 1
.endr

.code16
.global int_handler_common
int_handler_common:
    # 腾出ebp, edi, gs
    push    ebp
    push    edi
    mov     edi, gs
    push    edi

    # 切换栈到0:7b00，让gs:ebp指向int前栈顶
    mov     edi, ss
    mov     gs, edi
    mov     ebp, esp
    add     ebp, 0x10
    xor     edi, edi
    mov     ss, edi
    mov     esp, 0x7b00

    # 保存ds段，将ds段重置为0
    mov     edi, ds
    push    edi
    xor     edi, edi
    mov     ds, edi

    # 保存int时eax, ecx, edx, ebx, ebp, esi, edi, flags的值到新栈上
.att_syntax
    movw    %gs:0x4(%ebp), %di
    pushw   %di
    movl    %gs:-0xc(%ebp), %edi
    pushl   %edi
    pushl   %esi
    movl    %gs:-0x8(%ebp), %edi
    pushl   %edi
    pushl   %ebx
    pushl   %edx
    pushl   %ecx
    pushl   %eax
    movl    %gs:-0x4(%ebp), %edi
.intel_syntax noprefix

    # 压栈参数
    mov     esi, esp
    add     esi, 0x1c
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    sub     esi, 0x04
    push    esi
    push    edi

    calld   handler

    # 清除参数
    add     esp, 0x24

    # 恢复寄存器
.att_syntax
    popl    %eax
    popl    %ecx
    popl    %edx
    popl    %ebx
    popl    %edi
    movl    %edi, %gs:-0x8(%ebp)
    popl    %esi
    popl    %edi
    movl    %edi, %gs:-0xc(%ebp)
    popw    %di
    movw    %di, %gs:0x4(%ebp)
.intel_syntax noprefix

    # 恢复ds段
    pop     edi
    mov     ds, edi

    # 恢复栈
    mov     edi, gs
    mov     ss, edi
    mov     esp, ebp
    sub     esp, 0x10

    # 恢复ebp, edi, gs
    pop     edi
    mov     gs, edi
    pop     edi
    pop     ebp
    add     esp, 0x4
    iret

.att_syntax
.global cpy_to_es4
cpy_to_es4:
    push    %ebp
    mov     %esp, %ebp
    push    %ebx

    mov     0x8(%ebp), %edx
    mov     0xc(%ebp), %ecx
    xor     %eax, %eax

cpy_to_es4_chk:
    cmp     %eax, 0x10(%ebp)
    jbe     cpy_to_es4_ret
    mov     %ds:(%ecx,%eax,1), %ebx
    mov     %ebx, %es:(%edx,%eax,1)
    add     $0x4, %eax
    jmp     cpy_to_es4_chk

cpy_to_es4_ret:
    pop     %ebx
    pop     %ebp
    retl
.intel_syntax noprefix

.global end_of_virt_int_s
end_of_virt_int_s:
