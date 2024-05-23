use super::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use super::current_cpu_id;
use super::set_core_id_to_cpu_id;

unsafe extern "sysv64" fn switch_stack(core_id: usize, linux_sp: usize) -> i32 {
    let linux_tp = x86::msr::rdmsr(x86::msr::IA32_GS_BASE) as u64;
    let cpu_id = current_cpu_id();
    set_core_id_to_cpu_id(core_id, cpu_id);
    let per_cpu_array_ptr: usize = PER_CPU_ARRAY_PTR as usize + core_id as usize * PER_CPU_SIZE;
    let hv_sp = per_cpu_array_ptr + PER_CPU_SIZE - 8;
    let ret;
    core::arch::asm!("
        mov [rsi], {linux_tp}   // save gs_base to stack
        mov rcx, rsp
        mov rsp, {hv_sp}
        push rcx
        call {entry}
        pop rsp",
        entry = sym super::rust_entry_hv,
        linux_tp = in(reg) linux_tp,
        hv_sp = in(reg) hv_sp,
        in("rdi") core_id,
        in("rsi") linux_sp,
        lateout("rax") ret,
        out("rcx") _,
    );
    x86::msr::wrmsr(x86::msr::IA32_GS_BASE, linux_tp);
    ret
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> i32 {
    core::arch::asm!("
        // rip is pushed
        cli
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15
        push 0  // skip gs_base

        mov rsi, rsp
        call {0}

        pop r15 // skip gs_base
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        ret
        // rip will pop when return",
        sym switch_stack,
        options(noreturn),
    );
}
