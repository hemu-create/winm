pub fn enter(entry: u64, stack: u64, context: u64) -> ! {
    let spsr: u64 = 0x3c5; // EL1h, DAIF masked

    unsafe {
        core::arch::asm!(
            "msr SPSR_EL2, {spsr}",
            "msr ELR_EL2, {entry}",
            "msr SP_EL1, {stack}",
            "mov x0, {context}",
            "eret",
            spsr = in(reg) spsr,
            entry = in(reg) entry,
            stack = in(reg) stack,
            context = in(reg) context,
            options(noreturn),
        );
    }
}
