use alloc::string::String;

pub struct VmConfig {
    name: String,
    nr_vcpu: u32,
    cpu_affinity: usize,
}