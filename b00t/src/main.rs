#![no_std]
#![no_main]
#![allow(dead_code)]

use core::panic::PanicInfo;

mod acpi;
mod aic;
mod el1;
mod fdt;
mod gic;
mod handoff;
mod hvc;
mod memmap;
mod mmu;
mod platform;
mod psci;
mod stage2;
mod uart;

core::arch::global_asm!(include_str!("boot.S"));
core::arch::global_asm!(include_str!("vectors.S"));

extern "C" {
    fn el1_test();
}

static mut GIC_PTR: *mut gic::VirtualGic = core::ptr::null_mut();
static mut PSCI_PTR: *mut psci::PsciState = core::ptr::null_mut();

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rust_main(fdt_ptr: *const u8) -> ! {
    let fdt = match unsafe { fdt::Fdt::from_ptr(fdt_ptr) } {
        Some(fdt) => fdt,
        None => park(),
    };

    let uart = uart::Uart::from_fdt(&fdt).unwrap_or_else(uart::Uart::none);

    uart.puts("b00t: WinM stage 2\n");
    uart.puts("b00t: fdt=");
    uart.put_hex(fdt_ptr as u64);
    uart.puts(" size=");
    uart.put_hex(fdt.total_size() as u64);
    uart.puts("\n");

    if let Some(root) = fdt.find_node("/") {
        if let Some(compatible) = fdt.property_str(root, "compatible") {
            uart.puts("b00t: compatible=");
            uart.puts(compatible);
            uart.puts("\\n");

            let soc = platform::Soc::from_compatible(compatible);
            uart.puts("b00t: soc=");
            uart.puts(soc.name());
            uart.puts("\\n");
            uart.puts("b00t: supported=");
            uart.puts(if platform::is_supported(soc) {
                "yes"
            } else {
                "no"
            });
            uart.puts("\\n");
        }
    }

    if let Some(memory) = fdt.find_node("/memory") {
        if let Some(address) = fdt.reg_addr(memory) {
            uart.puts("b00t: memory=");
            uart.put_hex(address);
            if let Some(size) = fdt.reg_size(memory) {
                uart.puts(" size=");
                uart.put_hex(size);
            }
            uart.puts("\n");
        }
    }

    let mut framebuffer_base = 0u64;
    let mut framebuffer_width = 0u32;
    let mut framebuffer_height = 0u32;
    let mut framebuffer_stride = 0u32;
    let framebuffer_format = 0u32;

    if let Some(framebuffer) = fdt.find_node("/chosen/framebuffer") {
        framebuffer_base = fdt.reg_addr(framebuffer).unwrap_or(0);
        framebuffer_width = fdt.property_u32(framebuffer, "width").unwrap_or(0);
        framebuffer_height = fdt.property_u32(framebuffer, "height").unwrap_or(0);
        framebuffer_stride = fdt.property_u32(framebuffer, "stride").unwrap_or(0);

        uart.puts("b00t: fb=");
        uart.put_hex(framebuffer_base);
        uart.puts(" width=");
        uart.put_hex(framebuffer_width as u64);
        uart.puts(" height=");
        uart.put_hex(framebuffer_height as u64);
        uart.puts(" stride=");
        uart.put_hex(framebuffer_stride as u64);
        if let Some(format) = fdt.property_str(framebuffer, "format") {
            uart.puts(" format=");
            uart.puts(format);
        }
        uart.puts("\n");
    }

    if let Some(aic_node) = fdt
        .find_node("/arm-io/aic")
        .or_else(|| fdt.find_compatible("apple,t8122-aic3"))
    {
        uart.puts("b00t: aic=");
        uart.put_hex(fdt.reg_addr(aic_node).unwrap_or(0));
        if let Some(compatible) = fdt.property_str(aic_node, "compatible") {
            uart.puts(" compatible=");
            uart.puts(compatible);
        }
        uart.puts("\\n");
    }

    if let Some(aic) = aic::Aic::from_fdt(&fdt) {
        uart.puts("b00t: aic-base=");
        uart.put_hex(aic.base as u64);
        uart.puts(" event-offset=");
        uart.put_hex(aic.event_offset as u64);
        uart.puts("\\n");
        aic.mask_all();
        uart.puts("b00t: aic-masked\\n");
    }

    uart.puts("b00t: uart=");
    uart.put_hex(uart.base() as u64);
    uart.puts("\\n");

    let memory_map = memmap::collect(&fdt);
    uart.puts("b00t: memory-regions=");
    uart.put_hex(memory_map.count as u64);
    uart.puts("\\n");
    for index in 0..memory_map.count {
        let region = &memory_map.regions[index];
        uart.puts("  region base=");
        uart.put_hex(region.base);
        uart.puts(" size=");
        uart.put_hex(region.size);
        uart.puts("\\n");
    }

    let mut header = [0u8; acpi::TABLE_HEADER_LEN];
    acpi::init_table_header(
        &mut header,
        b"FACP",
        acpi::TABLE_HEADER_LEN as u32,
        6,
        b"WINM  ",
        b"WINM    ",
    );
    uart.puts("b00t: acpi_selftest=");
    uart.puts(if acpi::table_header_valid(&header) {
        "ok"
    } else {
        "failed"
    });
    uart.puts("\n");

    let mut rsdp = [0u8; acpi::RSDP_LEN];
    let mut xsdt = [0u8; 84];
    let mut fadt = [0u8; 276];
    let mut madt = [0u8; 44 + 80 + 24];
    let mut gtdt = [0u8; 96];
    let mut spcr = [0u8; 80];
    let mut dbg2 = [0u8; 128];

    let fadt_len = acpi::build_fadt(&mut fadt, 0);
    let madt_len = acpi::build_madt(&mut madt);
    let gtdt_len = acpi::build_gtdt(&mut gtdt);
    let spcr_len = acpi::build_spcr(&mut spcr, uart.base() as u64);
    let dbg2_len = acpi::build_dbg2(&mut dbg2, uart.base() as u64);
    let xsdt_entries = [
        fadt.as_ptr() as u64,
        madt.as_ptr() as u64,
        gtdt.as_ptr() as u64,
        spcr.as_ptr() as u64,
        dbg2.as_ptr() as u64,
    ];
    let xsdt_len = acpi::build_xsdt(&mut xsdt, &xsdt_entries);
    acpi::init_rsdp(&mut rsdp, xsdt.as_ptr() as u64);

    uart.puts("b00t: acpi rsdp=");
    uart.put_hex(rsdp.as_ptr() as u64);
    uart.puts(" xsdt=");
    uart.put_hex(xsdt_len as u64);
    uart.puts(" fadt=");
    uart.put_hex(fadt_len as u64);
    uart.puts(" madt=");
    uart.put_hex(madt_len as u64);
    uart.puts(" gtdt=");
    uart.put_hex(gtdt_len as u64);
    uart.puts(" spcr=");
    uart.put_hex(spcr_len as u64);
    uart.puts(" dbg2=");
    uart.put_hex(dbg2_len as u64);
    uart.puts("\\n");

    let mut psci_state = psci::PsciState::new();
    unsafe {
        PSCI_PTR = &mut psci_state;
    }
    let psci_version = psci::handle(&mut psci_state, psci::PSCI_VERSION, [0; 4]);
    uart.puts("b00t: psci=");
    uart.put_hex(psci_version as u64);
    uart.puts(" cpus=");
    uart.put_hex(psci_state.cpu_count as u64);
    uart.puts("\\n");

    let hvc_version = hvc::handle(hvc::HVC_GET_VERSION, 0, 0, 0, 0);
    uart.puts("b00t: hvc=");
    uart.put_hex(hvc_version as u64);
    uart.puts("\\n");

    let mut gic = gic::VirtualGic::new();
    gic.enable();
    unsafe {
        GIC_PTR = &mut gic;
    }
    uart.puts("b00t: gicv2=");
    uart.puts(if gic.is_enabled() { "on" } else { "off" });
    uart.puts("\n");

    let handoff = handoff::Handoff::new(
        fdt_ptr as u64,
        uart.base() as u64,
        framebuffer_base,
        framebuffer_width,
        framebuffer_height,
        framebuffer_stride,
        framebuffer_format,
        0,
    );

    uart.puts("b00t: handoff=");
    uart.put_hex(handoff.magic);
    uart.puts(" version=");
    uart.put_hex(handoff.version as u64);
    uart.puts("\n");

    mmu::build_all_device();
    for index in 0..memory_map.count {
        let region = &memory_map.regions[index];
        if region.size > 0 && region.kind == memmap::MemoryKind::Ram {
            mmu::mark_normal(region.base, region.size);
        }
    }
    uart.puts("b00t: mmu-l1=");
    uart.put_hex(mmu::table_base() as u64);
    uart.puts("\\n");
    uart.puts("b00t: enabling mmu\\n");
    mmu::enable();
    uart.puts("b00t: mmu=on\\n");

    stage2::build_identity();
    uart.puts("b00t: stage2-l1=");
    uart.put_hex(stage2::table_base() as u64);
    uart.puts("\\n");
    stage2::enable();
    uart.puts("b00t: stage2=on\\n");

    uart.puts("b00t: entering el1\\n");
    el1::enter(el1_test as *const () as usize as u64, 0, 0);
}

#[no_mangle]
#[allow(static_mut_refs)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rust_exception_handler(
    vector: u64,
    esr: u64,
    far: u64,
    elr: u64,
    regs: *mut u64,
) {
    let ec = (esr >> 26) & 0x3f;
    if ec == 0x16 {
        let function = unsafe { *regs.add(0) };
        let args = [
            unsafe { *regs.add(1) },
            unsafe { *regs.add(2) },
            unsafe { *regs.add(3) },
            unsafe { *regs.add(4) },
        ];

        let result = if function >= 0x8400_0000 {
            unsafe {
                if PSCI_PTR.is_null() {
                    psci::PSCI_NOT_SUPPORTED
                } else {
                    psci::handle(&mut *PSCI_PTR, function as u32, args)
                }
            }
        } else {
            hvc::handle(function, args[0], args[1], args[2], args[3])
        };

        unsafe {
            *regs.add(0) = result as u64;
            core::arch::asm!("msr ELR_EL2, {}", in(reg) elr + 4, options(nostack, preserves_flags));
        }
        return;
    }

    if ec == 0x24 || ec == 0x25 {
        let isv = esr & (1 << 24) != 0;
        let wnr = (esr >> 6) & 1 != 0;
        let rt = ((esr >> 16) & 0x1f) as usize;
        let ipa = far;

        let mut handled = false;
        unsafe {
            if !GIC_PTR.is_null() {
                let gic = &mut *GIC_PTR;
                if ipa >= gic::GICD_BASE_DEFAULT as u64
                    && ipa < (gic::GICD_BASE_DEFAULT + gic::GICD_SIZE) as u64
                {
                    let offset = (ipa - gic::GICD_BASE_DEFAULT as u64) as usize;
                    if isv {
                        if wnr {
                            let value = *regs.add(rt) as u32;
                            let mut value = value;
                            gic.handle_distributor(offset, true, &mut value);
                        } else {
                            let mut value = 0u32;
                            gic.handle_distributor(offset, false, &mut value);
                            *regs.add(rt) = value as u64;
                        }
                    }
                    handled = true;
                } else if ipa >= gic::GICC_BASE_DEFAULT as u64
                    && ipa < (gic::GICC_BASE_DEFAULT + gic::GICC_SIZE) as u64
                {
                    let offset = (ipa - gic::GICC_BASE_DEFAULT as u64) as usize;
                    if isv {
                        if wnr {
                            let value = *regs.add(rt) as u32;
                            let mut value = value;
                            gic.handle_cpu_interface(offset, true, &mut value);
                        } else {
                            let mut value = 0u32;
                            gic.handle_cpu_interface(offset, false, &mut value);
                            *regs.add(rt) = value as u64;
                        }
                    }
                    handled = true;
                }
            }
        }

        unsafe {
            uart::early_puts(if handled {
                "b00t: gic mmio\\n"
            } else {
                "b00t: stage2 abort\\n"
            });
            core::arch::asm!("msr ELR_EL2, {}", in(reg) elr + 4, options(nostack, preserves_flags));
        }
        return;
    }

    unsafe {
        uart::early_puts("b00t: exception vector=");
        uart::early_put_hex(vector);
        uart::early_puts(" esr=");
        uart::early_put_hex(esr);
        uart::early_puts(" far=");
        uart::early_put_hex(far);
        uart::early_puts(" elr=");
        uart::early_put_hex(elr);
        uart::early_puts("\\n");
    }
    park()
}

fn park() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    park()
}
