pub const GICD_BASE_DEFAULT: usize = 0x0800_0000;
pub const GICC_BASE_DEFAULT: usize = 0x0801_0000;
pub const GICD_SIZE: usize = 0x1000;
pub const GICC_SIZE: usize = 0x2000;

pub const GICD_CTLR: usize = 0x000;
pub const GICD_ISENABLER: usize = 0x100;
pub const GICD_ICENABLER: usize = 0x180;
pub const GICD_ISPENDR: usize = 0x200;
pub const GICD_ICPENDR: usize = 0x280;
pub const GICD_IPRIORITYR: usize = 0x400;
pub const GICD_ITARGETSR: usize = 0x800;
pub const GICD_ICFGR: usize = 0xc00;
pub const GICD_SGIR: usize = 0xf00;

pub const GICC_CTLR: usize = 0x000;
pub const GICC_PMR: usize = 0x004;
pub const GICC_BPR: usize = 0x008;
pub const GICC_IAR: usize = 0x00c;
pub const GICC_EOIR: usize = 0x010;
pub const GICC_RPR: usize = 0x014;
pub const GICC_HPPIR: usize = 0x018;
pub const GICC_DIR: usize = 0x1000;

pub const GIC_INTID_TIMER_VIRT: u32 = 27;
pub const GIC_INTID_TIMER_PHYS: u32 = 30;
pub const GIC_INTID_SPURIOUS: u32 = 1023;
pub const MAX_INTIDS: usize = 1024;

#[derive(Clone, Copy)]
pub struct VirtualGic {
    pub enabled: bool,
    pub distributor_enable: bool,
    pub cpu_enable: bool,
    pub pmr: u8,
    pub bpr: u8,
    pub iar: u32,
    pub enable: [u8; MAX_INTIDS / 8],
    pub pending: [u8; MAX_INTIDS / 8],
    pub active: [u8; MAX_INTIDS / 8],
    pub priority: [u8; MAX_INTIDS],
    pub target: [u8; MAX_INTIDS],
    pub config: [u8; MAX_INTIDS / 4],
}

impl VirtualGic {
    pub fn new() -> Self {
        let mut gic = VirtualGic {
            enabled: false,
            distributor_enable: false,
            cpu_enable: false,
            pmr: 0xff,
            bpr: 0,
            iar: GIC_INTID_SPURIOUS,
            enable: [0; MAX_INTIDS / 8],
            pending: [0; MAX_INTIDS / 8],
            active: [0; MAX_INTIDS / 8],
            priority: [0; MAX_INTIDS],
            target: [0; MAX_INTIDS],
            config: [0; MAX_INTIDS / 4],
        };

        for index in 0..MAX_INTIDS {
            gic.priority[index] = 0xa0;
            gic.target[index] = 0x01;
        }
        gic
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn bit(array: &mut [u8], index: usize, value: bool) {
        let byte = index / 8;
        let mask = 1u8 << (index % 8);
        if value {
            array[byte] |= mask;
        } else {
            array[byte] &= !mask;
        }
    }

    fn get_bit(array: &[u8], index: usize) -> bool {
        let byte = index / 8;
        let mask = 1u8 << (index % 8);
        array[byte] & mask != 0
    }

    pub fn set_pending(&mut self, intid: u32) {
        if (intid as usize) < MAX_INTIDS {
            Self::bit(&mut self.pending, intid as usize, true);
        }
    }

    pub fn clear_pending(&mut self, intid: u32) {
        if (intid as usize) < MAX_INTIDS {
            Self::bit(&mut self.pending, intid as usize, false);
        }
    }

    pub fn is_pending(&self, intid: u32) -> bool {
        if (intid as usize) < MAX_INTIDS {
            Self::get_bit(&self.pending, intid as usize)
        } else {
            false
        }
    }

    pub fn handle_distributor(&mut self, offset: usize, write: bool, value: &mut u32) -> bool {
        if offset >= GICD_SIZE {
            return false;
        }

        match offset {
            GICD_CTLR => {
                if write {
                    self.distributor_enable = *value & 1 != 0;
                } else {
                    *value = u32::from(self.distributor_enable);
                }
            }
            offset if (GICD_ISENABLER..GICD_ISENABLER + 0x80).contains(&offset) => {
                let index = (offset - GICD_ISENABLER) / 4;
                let base = index * 32;
                if write {
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if *value & (1 << bit) != 0 && (intid as usize) < MAX_INTIDS {
                            Self::bit(&mut self.enable, intid as usize, true);
                        }
                    }
                } else {
                    let mut result = 0u32;
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if (intid as usize) < MAX_INTIDS
                            && Self::get_bit(&self.enable, intid as usize)
                        {
                            result |= 1 << bit;
                        }
                    }
                    *value = result;
                }
            }
            offset if (GICD_ICENABLER..GICD_ICENABLER + 0x80).contains(&offset) => {
                let index = (offset - GICD_ICENABLER) / 4;
                let base = index * 32;
                if write {
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if *value & (1 << bit) != 0 && (intid as usize) < MAX_INTIDS {
                            Self::bit(&mut self.enable, intid as usize, false);
                        }
                    }
                } else {
                    let mut result = 0u32;
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if (intid as usize) < MAX_INTIDS
                            && Self::get_bit(&self.enable, intid as usize)
                        {
                            result |= 1 << bit;
                        }
                    }
                    *value = result;
                }
            }
            offset if (GICD_ISPENDR..GICD_ISPENDR + 0x80).contains(&offset) => {
                let index = (offset - GICD_ISPENDR) / 4;
                let base = index * 32;
                if write {
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if *value & (1 << bit) != 0 {
                            self.set_pending(intid);
                        }
                    }
                } else {
                    let mut result = 0u32;
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if self.is_pending(intid) {
                            result |= 1 << bit;
                        }
                    }
                    *value = result;
                }
            }
            offset if (GICD_ICPENDR..GICD_ICPENDR + 0x80).contains(&offset) => {
                let index = (offset - GICD_ICPENDR) / 4;
                let base = index * 32;
                if write {
                    for bit in 0..32u32 {
                        let intid = base as u32 + bit;
                        if *value & (1 << bit) != 0 {
                            self.clear_pending(intid);
                        }
                    }
                } else {
                    *value = 0;
                }
            }
            offset if (GICD_IPRIORITYR..GICD_IPRIORITYR + 0x400).contains(&offset) => {
                let index = offset - GICD_IPRIORITYR;
                if write {
                    self.priority[index] = (*value & 0xff) as u8;
                } else {
                    *value = self.priority[index] as u32;
                }
            }
            offset if (GICD_ITARGETSR..GICD_ITARGETSR + 0x400).contains(&offset) => {
                let index = offset - GICD_ITARGETSR;
                if write {
                    self.target[index] = (*value & 0xff) as u8;
                } else {
                    *value = self.target[index] as u32;
                }
            }
            offset if (GICD_ICFGR..GICD_ICFGR + 0x200).contains(&offset) => {
                let index = (offset - GICD_ICFGR) / 4;
                let base = index * 16;
                if write {
                    for i in 0..16usize {
                        let shift = i * 2;
                        self.config[base + i] = ((*value >> shift) & 0x3) as u8;
                    }
                } else {
                    let mut result = 0u32;
                    for i in 0..16usize {
                        let shift = i * 2;
                        result |= (self.config[base + i] as u32 & 0x3) << shift;
                    }
                    *value = result;
                }
            }
            GICD_SGIR => {
                if write {
                    // Software-generated interrupt: set pending for requested targets.
                    let intid = *value & 0xf;
                    let target_list = (*value >> 16) & 0xff;
                    let filter = (*value >> 24) & 0x3;
                    if filter == 0 {
                        for cpu in 0..8u32 {
                            if target_list & (1 << cpu) != 0 {
                                self.set_pending(intid);
                            }
                        }
                    } else {
                        self.set_pending(intid);
                    }
                }
            }
            _ => return false,
        }

        true
    }

    pub fn handle_cpu_interface(&mut self, offset: usize, write: bool, value: &mut u32) -> bool {
        if offset >= GICC_SIZE {
            return false;
        }

        match offset {
            GICC_CTLR => {
                if write {
                    self.cpu_enable = *value & 1 != 0;
                } else {
                    *value = u32::from(self.cpu_enable);
                }
            }
            GICC_PMR => {
                if write {
                    self.pmr = (*value & 0xff) as u8;
                } else {
                    *value = self.pmr as u32;
                }
            }
            GICC_BPR => {
                if write {
                    self.bpr = (*value & 0xff) as u8;
                } else {
                    *value = self.bpr as u32;
                }
            }
            GICC_IAR => {
                if write {
                    // Reads are the meaningful operation; ignore writes.
                } else {
                    *value = self.iar;
                }
            }
            GICC_EOIR => {
                if write {
                    let intid = *value & 0x3ff;
                    if (intid as usize) < MAX_INTIDS {
                        Self::bit(&mut self.active, intid as usize, false);
                        self.clear_pending(intid);
                    }
                    self.iar = GIC_INTID_SPURIOUS;
                }
            }
            GICC_RPR => {
                if !write {
                    *value = self.pmr as u32;
                }
            }
            GICC_HPPIR => {
                if !write {
                    *value = self.iar;
                }
            }
            GICC_DIR => {
                if write {
                    let intid = *value & 0x3ff;
                    if (intid as usize) < MAX_INTIDS {
                        Self::bit(&mut self.active, intid as usize, false);
                    }
                }
            }
            _ => return false,
        }

        true
    }

    pub fn acknowledge(&mut self) -> u32 {
        for intid in 0..MAX_INTIDS as u32 {
            if self.is_pending(intid) && Self::get_bit(&self.enable, intid as usize) {
                Self::bit(&mut self.active, intid as usize, true);
                self.iar = intid;
                return intid;
            }
        }
        self.iar = GIC_INTID_SPURIOUS;
        GIC_INTID_SPURIOUS
    }
}
