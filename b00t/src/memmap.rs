#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Ram,
    Reserved,
    Framebuffer,
}

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub kind: MemoryKind,
}

pub const MAX_REGIONS: usize = 64;

pub struct MemoryMap {
    pub regions: [MemoryRegion; MAX_REGIONS],
    pub count: usize,
}

impl MemoryMap {
    pub fn empty() -> Self {
        MemoryMap {
            regions: [MemoryRegion {
                base: 0,
                size: 0,
                kind: MemoryKind::Ram,
            }; MAX_REGIONS],
            count: 0,
        }
    }

    pub fn add(&mut self, base: u64, size: u64, kind: MemoryKind) -> bool {
        if self.count >= MAX_REGIONS || size == 0 {
            return false;
        }
        self.regions[self.count] = MemoryRegion { base, size, kind };
        self.count += 1;
        true
    }
}

pub fn collect(fdt: &crate::fdt::Fdt<'_>) -> MemoryMap {
    let mut map = MemoryMap::empty();

    if let Some(memory) = fdt.find_node("/memory") {
        let (pairs, count) = fdt.reg_pairs(memory);
        for (base, size) in pairs.iter().take(count) {
            map.add(*base, *size, MemoryKind::Ram);
        }
    }

    if let Some(reserved) = fdt.find_node("/reserved-memory") {
        let (children, child_count) = fdt.child_nodes(reserved);
        for child in children.iter().take(child_count) {
            let (pairs, count) = fdt.reg_pairs(*child);
            for (base, size) in pairs.iter().take(count) {
                map.add(*base, *size, MemoryKind::Reserved);
            }
        }
    }

    if let Some(framebuffer) = fdt.find_node("/chosen/framebuffer") {
        if let (Some(base), Some(size)) = (fdt.reg_addr(framebuffer), fdt.reg_size(framebuffer)) {
            map.add(base, size, MemoryKind::Framebuffer);
        }
    }

    map
}
