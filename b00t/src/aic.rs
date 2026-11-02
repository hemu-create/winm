use crate::fdt::Fdt;

pub const AIC_EVENT_OFFSET_DEFAULT: usize = 0x2004;

pub struct Aic {
    pub base: usize,
    pub compatible: &'static str,
    pub event_offset: usize,
}

impl Aic {
    pub fn from_fdt(fdt: &Fdt<'_>) -> Option<Self> {
        let node = fdt
            .find_node("/arm-io/aic")
            .or_else(|| fdt.find_compatible("apple,aic"))
            .or_else(|| fdt.find_compatible("apple,aic2"))
            .or_else(|| fdt.find_compatible("apple,t8122-aic3"))?;

        let base = fdt.reg_addr(node)? as usize;
        if base == 0 {
            return None;
        }

        let compatible = fdt.property_str(node, "compatible").unwrap_or("apple,aic");

        let compatible = if compatible.contains("aic3") {
            "apple,aic3"
        } else if compatible.contains("aic2") {
            "apple,aic2"
        } else {
            "apple,aic"
        };

        let event_offset = fdt
            .property_u32(node, "aic-iack-offset")
            .map(|value| value as usize)
            .unwrap_or(AIC_EVENT_OFFSET_DEFAULT);

        Some(Aic {
            base,
            compatible,
            event_offset,
        })
    }

    pub fn read_event(&self) -> u32 {
        if self.base == 0 {
            return 0;
        }
        unsafe { core::ptr::read_volatile((self.base + self.event_offset) as *const u32) }
    }

    pub fn mask_all(&self) {
        if self.compatible != "apple,aic" || self.base == 0 {
            return;
        }
        for index in 0..(0x400 / 32) {
            unsafe {
                core::ptr::write_volatile(
                    (self.base + 0x4100 + index * 4) as *mut u32,
                    0xffff_ffff,
                );
            }
        }
    }
}
