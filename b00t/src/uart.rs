use crate::fdt::Fdt;

const UTRSTAT: usize = 0x010;
const UTXH: usize = 0x020;
const UTRSTAT_TXBE: u32 = 1 << 1;

pub struct Uart {
    base: usize,
}

impl Uart {
    pub fn none() -> Self {
        Uart { base: 0 }
    }

    pub fn from_fdt(fdt: &Fdt<'_>) -> Option<Self> {
        let mut node_path = None;

        if let Some(chosen) = fdt.find_node("/chosen") {
            if let Some(path) = fdt
                .property_str(chosen, "stdout-path")
                .or_else(|| fdt.property_str(chosen, "linux,stdout-path"))
            {
                node_path = Some(path.split(':').next().unwrap_or(path));
            }
        }

        if node_path.is_none() {
            if let Some(aliases) = fdt.find_node("/aliases") {
                if let Some(path) = fdt.property_str(aliases, "serial0") {
                    node_path = Some(path);
                }
            }
        }

        let node = if let Some(path) = node_path {
            if path.starts_with('/') {
                fdt.find_node(path)
            } else if let Some(aliases) = fdt.find_node("/aliases") {
                fdt.property_str(aliases, path)
                    .and_then(|full_path| fdt.find_node(full_path))
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| fdt.find_compatible("apple,s5l-uart"))?;

        let base = fdt.reg_addr(node)? as usize;
        if base == 0 {
            return None;
        }

        Some(Uart { base })
    }

    pub fn base(&self) -> usize {
        self.base
    }

    pub fn putc(&self, c: u8) {
        if self.base == 0 {
            return;
        }

        while unsafe { core::ptr::read_volatile((self.base + UTRSTAT) as *const u32) }
            & UTRSTAT_TXBE
            == 0
        {}

        unsafe {
            core::ptr::write_volatile((self.base + UTXH) as *mut u32, c as u32);
        }
    }

    pub fn puts(&self, text: &str) {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.putc(b'\r');
            }
            self.putc(byte);
        }
    }

    pub fn put_hex(&self, value: u64) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.puts("0x");
        for shift in (0..64).step_by(4).rev() {
            let digit = ((value >> shift) & 0xf) as usize;
            self.putc(HEX[digit]);
        }
    }
}

pub const EARLY_UART_BASE: usize = 0x2a1200000;

pub unsafe fn early_putc(c: u8) {
    let base = EARLY_UART_BASE;
    while core::ptr::read_volatile((base + UTRSTAT) as *const u32) & UTRSTAT_TXBE == 0 {}
    core::ptr::write_volatile((base + UTXH) as *mut u32, c as u32);
}

pub unsafe fn early_puts(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' {
            early_putc(b'\r');
        }
        early_putc(byte);
    }
}

pub unsafe fn early_put_hex(value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    early_puts("0x");
    for shift in (0..64).step_by(4).rev() {
        let digit = ((value >> shift) & 0xf) as usize;
        early_putc(HEX[digit]);
    }
}
