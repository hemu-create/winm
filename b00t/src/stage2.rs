pub const L1_ENTRIES: usize = 512;

static mut STAGE2_L1: [u64; L1_ENTRIES] = [0; L1_ENTRIES];

pub fn table_base() -> usize {
    core::ptr::addr_of!(STAGE2_L1) as usize
}

pub fn build_identity() {
    for index in 0..L1_ENTRIES {
        let address = (index as u64) << 30;
        let value = address | (1 << 10) | 1;
        unsafe {
            let table = core::ptr::addr_of_mut!(STAGE2_L1);
            (*table)[index] = value;
        }
    }
}

pub fn enable() {
    let table = table_base() as u64;
    let vtcr: u64 = 25 | (1 << 6) | (1 << 8) | (1 << 10) | (3 << 12) | (4 << 16);

    unsafe {
        core::arch::asm!(
            "msr VTCR_EL2, {vtcr}",
            "msr VTTBR_EL2, {ttbr}",
            "isb",
            "mrs x0, HCR_EL2",
            "orr x0, x0, #1",
            "msr HCR_EL2, x0",
            "isb",
            vtcr = in(reg) vtcr,
            ttbr = in(reg) table,
            out("x0") _,
            options(nostack, preserves_flags),
        );
    }
}
