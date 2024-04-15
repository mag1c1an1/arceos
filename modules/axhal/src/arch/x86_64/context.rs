use core::{arch::asm, fmt};
use memory_addr::{PhysAddr, VirtAddr};

use x86_64::registers::rflags::RFlags;

use super::gdt::GdtStruct;

/// Saved registers when a trap (interrupt or exception) occurs.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Pushed by `trap.S`
    pub vector: u64,
    pub error_code: u64,

    // Pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,

    // Pushed by CPU when trap from ring-3
    pub user_rsp: u64,
    pub user_ss: u64,
}

#[cfg(feature = "monolithic")]
impl TrapFrame {
    pub fn new_user(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        Self {
            rdi: arg0 as _,
            rip: entry.as_usize() as _,
            cs: GdtStruct::UCODE64_SELECTOR.0 as _,
            rflags: RFlags::INTERRUPT_FLAG.bits(), // IOPL = 0, IF = 1
            user_rsp: ustack_top.as_usize() as _,
            user_ss: GdtStruct::UDATA_SELECTOR.0 as _,
            ..Default::default()
        }
    }

    pub const fn new_clone(&self, ustack_top: VirtAddr) -> Self {
        let mut tf = *self;
        // cs, user_ss are not pushed into TrapFrame in syscall_entry
        tf.cs = GdtStruct::UCODE64_SELECTOR.0 as _;
        tf.user_ss = GdtStruct::UDATA_SELECTOR.0 as _;
        tf.user_rsp = ustack_top.as_usize() as _;
        tf.rax = 0; // for child thread, clone returns 0
        tf
    }

    pub const fn new_fork(&self) -> Self {
        let mut tf = *self;
        // cs, user_ss are not pushed into TrapFrame in syscall_entry
        tf.cs = GdtStruct::UCODE64_SELECTOR.0 as _;
        tf.user_ss = GdtStruct::UDATA_SELECTOR.0 as _;
        tf.rax = 0; // for child process, fork returns 0
        tf
    }

    /// Whether the trap is from userspace.
    pub const fn is_user(&self) -> bool {
        self.cs & 0b11 == 3
    }

    /// 用于第一次进入应用程序时的初始化
    pub fn app_init_args(&mut self) {
        let user_sp = self.user_rsp;
        unsafe {
            // a0为参数个数
            // a1存储的是用户栈底，即argv
            self.rdi = *(user_sp as *const u64);
            self.rsi = *(user_sp as *const u64).add(1);
        }
    }

    pub unsafe fn exec(&self, kstack_top: VirtAddr) -> ! {
        info!(
            "user task start: entry={:#x}, ustack={:#x}, kstack={:#x}",
            self.rip,
            self.user_rsp,
            kstack_top.as_usize(),
        );
        crate::arch::disable_irqs();

        assert_eq!(crate::platform::kernel_stack_top(), kstack_top);

        asm!("
            mov     rsp, {tf}
            pop     rax
            pop     rcx
            pop     rdx
            pop     rbx
            pop     rbp
            pop     rsi
            pop     rdi
            pop     r8
            pop     r9
            pop     r10
            pop     r11
            pop     r12
            pop     r13
            pop     r14
            pop     r15
            add     rsp, 16     // pop vector, error_code
            swapgs
            iretq",
            tf = in(reg) self,
            options(noreturn),
        )
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct ContextSwitchFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rip: u64,
}

/// A 512-byte memory region for the FXSAVE/FXRSTOR instruction to save and
/// restore the x87 FPU, MMX, XMM, and MXCSR registers.
///
/// See <https://www.felixcloutier.com/x86/fxsave> for more details.
#[allow(missing_docs)]
#[repr(C, align(16))]
#[derive(Debug)]
pub struct FxsaveArea {
    pub fcw: u16,
    pub fsw: u16,
    pub ftw: u16,
    pub fop: u16,
    pub fip: u64,
    pub fdp: u64,
    pub mxcsr: u32,
    pub mxcsr_mask: u32,
    pub st: [u64; 16],
    pub xmm: [u64; 32],
    _padding: [u64; 12],
}

static_assertions::const_assert_eq!(core::mem::size_of::<FxsaveArea>(), 512);

/// Extended state of a task, such as FP/SIMD states.
pub struct ExtendedState {
    /// Memory region for the FXSAVE/FXRSTOR instruction.
    pub fxsave_area: FxsaveArea,
}

#[cfg(feature = "fp_simd")]
impl ExtendedState {
    #[inline]
    fn save(&mut self) {
        unsafe { core::arch::x86_64::_fxsave64(&mut self.fxsave_area as *mut _ as *mut u8) }
    }

    #[inline]
    fn restore(&self) {
        unsafe { core::arch::x86_64::_fxrstor64(&self.fxsave_area as *const _ as *const u8) }
    }

    const fn default() -> Self {
        let mut area: FxsaveArea = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        area.fcw = 0x37f;
        area.ftw = 0xffff;
        area.mxcsr = 0x1f80;
        Self { fxsave_area: area }
    }
}

impl fmt::Debug for ExtendedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtendedState")
            .field("fxsave_area", &self.fxsave_area)
            .finish()
    }
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
///
/// On x86_64, callee-saved registers are saved to the kernel stack by the
/// `PUSH` instruction. So that [`rsp`] is the `RSP` after callee-saved
/// registers are pushed, and [`kstack_top`] is the top of the kernel stack
/// (`RSP` before any push).
///
/// [`rsp`]: TaskContext::rsp
/// [`kstack_top`]: TaskContext::kstack_top
#[derive(Debug)]
pub struct TaskContext {
    /// The kernel stack top of the task.
    pub kstack_top: VirtAddr,
    /// `RSP` after all callee-saved registers are pushed.
    pub rsp: u64,
    pub cr3: u64,
    /// Extended states, i.e., FP/SIMD states.
    #[cfg(feature = "fp_simd")]
    pub ext_state: ExtendedState,
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        Self {
            kstack_top: VirtAddr::from(0),
            rsp: 0,
            cr3: 0,
            #[cfg(feature = "fp_simd")]
            ext_state: ExtendedState::default(),
        }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, page_table_root: PhysAddr) {
        unsafe {
            // x86_64 calling convention: the stack must be 16-byte aligned before
            // calling a function. That means when entering a new task (`ret` in `context_switch`
            // is executed), (stack pointer + 8) should be 16-byte aligned.
            let frame_ptr = (kstack_top.as_mut_ptr() as *mut u64).sub(1);
            let frame_ptr = (frame_ptr as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(
                frame_ptr,
                ContextSwitchFrame {
                    rip: entry as _,
                    ..Default::default()
                },
            );
            self.rsp = frame_ptr as u64;
        }
        self.kstack_top = kstack_top;
        self.cr3 = page_table_root.as_usize() as u64;
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        #[cfg(feature = "fp_simd")]
        {
            self.ext_state.save();
            next_ctx.ext_state.restore();
        }

        debug!(
            "switch to(), next kstack_top {:?} rsp {}",
            next_ctx.kstack_top, next_ctx.rsp
        );

        unsafe {
            // TODO: swtich tls

            // PerCpu::current()
            //     .arch_data()
            //     .as_mut()
            //     .set_kernel_stack_top(next_ctx.kstack_top);
            #[cfg(feature = "monolithic")]
            crate::platform::set_kernel_stack_top(next_ctx.kstack_top);

            context_switch(&mut self.rsp, &next_ctx.rsp)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_stack: &mut u64, _next_stack: &u64) {
    asm!(
        "
        push    rbp
        push    rbx
        push    r12
        push    r13
        push    r14
        push    r15
        mov     [rdi], rsp

        mov     rsp, [rsi]
        pop     r15
        pop     r14
        pop     r13
        pop     r12
        pop     rbx
        pop     rbp
        ret",
        options(noreturn),
    )
}
