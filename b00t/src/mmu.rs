pub const L1_ENTRIES: usize = 512;

static mut L1_TABLE: [u64; L1_ENTRIES] = [0; L1_ENTRIES];

pub fn table_base() -> usize {
    core::ptr::addr_of!(L1_TABLE) as usize
}

pub fn build_all_device() {
    for index in 0..L1_ENTRIES {
        let address = (index as u64) << 30;
        let value = address | (1 << 10) | (1 << 2) | 1;
        unsafe {
            let table = core::ptr::addr_of_mut!(L1_TABLE);
            (*table)[index] = value;
        }
    }
}

pub fn mark_normal(base: u64, size: u64) {
    if size == 0 {
        return;
    }
    let end = base.saturating_add(size);
    let mut addr = base & !((1u64 << 30) - 1);
    while addr < end {
        let index = (addr >> 30) as usize;
        if index < L1_ENTRIES {
            let block = addr | (1 << 10) | 1;
            unsafe {
                let table = core::ptr::addr_of_mut!(L1_TABLE);
                (*table)[index] = block;
            }
        }
        addr = addr.saturating_add(1 << 30);
    }
}

pub fn enable() {
    let table = table_base() as u64;
    let mair: u64 = 0x0000_0000_0000_00ff;
    let tcr: u64 = 25 | (1 << 8) | (1 << 10) | (3 << 12) | (4 << 32);

    let sctlr: u64 = 0x100d;
    unsafe {
        core::arch::asm!(
            "msr MAIR_EL2, {mair}",
            "msr TCR_EL2, {tcr}",
            "msr TTBR0_EL2, {ttbr}",
            "isb",
            "mrs x0, SCTLR_EL2",
            "orr x0, x0, {sctlr}",
            "msr SCTLR_EL2, x0",
            "isb",
            mair = in(reg) mair,
            tcr = in(reg) tcr,
            ttbr = in(reg) table,
            sctlr = in(reg) sctlr,
            out("x0") _,
            options(nostack, preserves_flags),
        );
    }
}
