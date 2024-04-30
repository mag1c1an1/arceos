use lazy_init::LazyInit;

use hypercraft::LinuxContext;

#[percpu::def_percpu]
static LINUX_CTX: LazyInit<LinuxContext> = LazyInit::new();

pub fn save_linux_context(linux_sp: usize) {
    let linux_ctx = unsafe { LINUX_CTX.current_ref_mut_raw() };

    linux_ctx.init_by(LinuxContext::load_from(linux_sp));
}

pub fn get_linux_context() -> &'static LinuxContext {
    unsafe { LINUX_CTX.current_ref_raw() }.try_get().unwrap()
}
