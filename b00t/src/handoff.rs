pub const HANDOFF_MAGIC: u64 = 0x5749_4e4d_5f42_3030;
pub const HANDOFF_VERSION: u32 = 1;

#[repr(C)]
pub struct Handoff {
    pub magic: u64,
    pub version: u32,
    pub reserved: u32,
    pub fdt: u64,
    pub uart_base: u64,
    pub framebuffer_base: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_stride: u32,
    pub framebuffer_format: u32,
    pub acpi_rsdp: u64,
}

impl Handoff {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fdt: u64,
        uart_base: u64,
        framebuffer_base: u64,
        framebuffer_width: u32,
        framebuffer_height: u32,
        framebuffer_stride: u32,
        framebuffer_format: u32,
        acpi_rsdp: u64,
    ) -> Self {
        Handoff {
            magic: HANDOFF_MAGIC,
            version: HANDOFF_VERSION,
            reserved: 0,
            fdt,
            uart_base,
            framebuffer_base,
            framebuffer_width,
            framebuffer_height,
            framebuffer_stride,
            framebuffer_format,
            acpi_rsdp,
        }
    }
}
