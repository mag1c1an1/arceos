use alloc::vec::Vec;
use bitvec::order::Lsb0;
use bitvec::slice::IterOnes;
use bitvec::vec::BitVec;

#[derive(Debug, Clone)]
pub struct CpuSet {
    bitset: BitVec,
}

impl CpuSet {
    /// Creates a new `CpuSet` with all CPUs included.
    pub fn new_full() -> Self {
        let num_cpus = axconfig::SMP;
        let mut bitset = BitVec::with_capacity(num_cpus);
        bitset.resize(num_cpus, true);
        Self { bitset }
    }

    /// Creates a new `CpuSet` with no CPUs included.
    pub fn new_empty() -> Self {
        let num_cpus = axconfig::SMP;
        let mut bitset = BitVec::with_capacity(num_cpus);
        bitset.resize(num_cpus as usize, false);
        Self { bitset }
    }

    /// Adds a CPU with identifier `cpu_id` to the `CpuSet`.
    pub fn add(&mut self, cpu_id: usize) {
        self.bitset.set(cpu_id, true);
    }

    /// Adds multiple CPUs from `cpu_ids` to the `CpuSet`.
    pub fn add_from_vec(&mut self, cpu_ids: Vec<usize>) {
        for cpu_id in cpu_ids {
            self.add(cpu_id)
        }
    }

    /// Adds all available CPUs to the `CpuSet`.
    pub fn add_all(&mut self) {
        self.bitset.fill(true);
    }

    /// Removes a CPU with identifier `cpu_id` from the `CpuSet`.
    pub fn remove(&mut self, cpu_id: usize) {
        self.bitset.set(cpu_id, false);
    }

    /// Removes multiple CPUs from `cpu_ids` from the `CpuSet`.
    pub fn remove_from_vec(&mut self, cpu_ids: Vec<usize>) {
        for cpu_id in cpu_ids {
            self.remove(cpu_id);
        }
    }

    /// Clears the `CpuSet`, removing all CPUs.
    pub fn clear(&mut self) {
        self.bitset.fill(false);
    }

    /// Checks if the `CpuSet` contains a specific CPU.
    pub fn contains(&self, cpu_id: usize) -> bool {
        self.bitset.get(cpu_id).as_deref() == Some(&true)
    }

    /// Returns an iterator over the set CPUs.
    pub fn iter(&self) -> IterOnes<'_, usize, Lsb0> {
        self.bitset.iter_ones()
    }
}
