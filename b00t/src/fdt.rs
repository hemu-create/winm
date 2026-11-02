const FDT_MAGIC: u32 = 0xd00d_feed;
const FDT_BEGIN_NODE: u32 = 1;
const FDT_END_NODE: u32 = 2;
const FDT_PROP: u32 = 3;
const FDT_NOP: u32 = 4;
const FDT_END: u32 = 9;
const MAX_FDT_SIZE: usize = 64 * 1024 * 1024;

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

pub struct Fdt<'a> {
    data: &'a [u8],
    off_struct: usize,
    size_struct: usize,
    off_strings: usize,
    size_strings: usize,
    address_cells: u8,
    size_cells: u8,
}

impl<'a> Fdt<'a> {
    pub unsafe fn from_ptr(ptr: *const u8) -> Option<Fdt<'a>> {
        if ptr.is_null() {
            return None;
        }

        let magic = u32::from_be(core::ptr::read_unaligned(ptr as *const u32));
        if magic != FDT_MAGIC {
            return None;
        }

        let total_size = u32::from_be(core::ptr::read_unaligned(ptr.add(4) as *const u32)) as usize;
        if !(40..=MAX_FDT_SIZE).contains(&total_size) {
            return None;
        }

        let data = core::slice::from_raw_parts(ptr, total_size);
        let off_struct = u32::from_be_bytes(data[8..12].try_into().ok()?) as usize;
        let off_strings = u32::from_be_bytes(data[12..16].try_into().ok()?) as usize;
        let size_struct = u32::from_be_bytes(data[36..40].try_into().ok()?) as usize;
        let size_strings = u32::from_be_bytes(data[32..36].try_into().ok()?) as usize;

        if off_struct + size_struct > total_size || off_strings + size_strings > total_size {
            return None;
        }

        let mut fdt = Fdt {
            data,
            off_struct,
            size_struct,
            off_strings,
            size_strings,
            address_cells: 2,
            size_cells: 2,
        };

        if let Some(root) = fdt.find_node("/") {
            if let Some(value) = fdt.property_u32(root, "#address-cells") {
                if value > 0 && value <= 4 {
                    fdt.address_cells = value as u8;
                }
            }
            if let Some(value) = fdt.property_u32(root, "#size-cells") {
                if value <= 4 {
                    fdt.size_cells = value as u8;
                }
            }
        }

        Some(fdt)
    }

    pub fn total_size(&self) -> usize {
        self.data.len()
    }

    pub fn address_cells(&self) -> u8 {
        self.address_cells
    }

    pub fn size_cells(&self) -> u8 {
        self.size_cells
    }

    fn u32_at(&self, off: usize) -> Option<u32> {
        let end = off.checked_add(4)?;
        if end > self.data.len() {
            return None;
        }
        Some(u32::from_be_bytes(self.data[off..end].try_into().ok()?))
    }

    fn u64_at(&self, off: usize) -> Option<u64> {
        let end = off.checked_add(8)?;
        if end > self.data.len() {
            return None;
        }
        Some(u64::from_be_bytes(self.data[off..end].try_into().ok()?))
    }

    fn cstr_struct(&self, off: usize) -> Option<&'a [u8]> {
        let limit = self.off_struct.checked_add(self.size_struct)?;
        self.cstr_at(off, limit)
    }

    fn cstr_strings(&self, off: usize) -> Option<&'a [u8]> {
        let limit = self.off_strings.checked_add(self.size_strings)?;
        self.cstr_at(off, limit)
    }

    fn cstr_at(&self, off: usize, limit: usize) -> Option<&'a [u8]> {
        if off >= limit || limit > self.data.len() {
            return None;
        }
        let bytes = &self.data[off..limit];
        let len = bytes.iter().position(|&b| b == 0)?;
        Some(&bytes[..len])
    }

    fn skip_prop(&self, off: &mut usize) -> Option<()> {
        let struct_end = self.off_struct.checked_add(self.size_struct)?;
        if *off + 8 > struct_end {
            return None;
        }
        let len = self.u32_at(*off)? as usize;
        *off += 8;
        *off = off.checked_add(align4(len))?;
        if *off > struct_end {
            return None;
        }
        Some(())
    }

    pub fn find_node(&self, path: &str) -> Option<usize> {
        if path.len() >= 256 {
            return None;
        }

        let struct_end = self.off_struct.checked_add(self.size_struct)?;
        let mut off = self.off_struct;
        let mut depth = 0usize;
        let mut path_len = 0usize;
        let mut path_buf = [0u8; 256];
        let mut saved = [0usize; 32];

        loop {
            if off + 4 > struct_end {
                return None;
            }
            let token = self.u32_at(off)?;
            off += 4;

            match token {
                FDT_BEGIN_NODE => {
                    if depth >= saved.len() {
                        return None;
                    }
                    let name = self.cstr_struct(off)?;
                    off = off.checked_add(align4(name.len() + 1))?;
                    let after_name = off;

                    let old_len = path_len;
                    saved[depth] = old_len;

                    if depth == 0 {
                        path_buf[0] = b'/';
                        path_len = 1;
                        if !name.is_empty() {
                            if path_len + name.len() > path_buf.len() {
                                return None;
                            }
                            path_buf[path_len..path_len + name.len()].copy_from_slice(name);
                            path_len += name.len();
                        }
                    } else {
                        if path_len > 1 {
                            if path_len + 1 > path_buf.len() {
                                return None;
                            }
                            path_buf[path_len] = b'/';
                            path_len += 1;
                        }
                        if path_len + name.len() > path_buf.len() {
                            return None;
                        }
                        path_buf[path_len..path_len + name.len()].copy_from_slice(name);
                        path_len += name.len();
                    }

                    if path_len == path.len() && &path_buf[..path_len] == path.as_bytes() {
                        return Some(after_name);
                    }

                    depth += 1;
                }
                FDT_END_NODE => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    path_len = saved[depth];
                }
                FDT_PROP => {
                    self.skip_prop(&mut off)?;
                }
                FDT_NOP => {}
                FDT_END => break,
                _ => return None,
            }
        }

        None
    }

    pub fn find_compatible(&self, compatible: &str) -> Option<usize> {
        let struct_end = self.off_struct.checked_add(self.size_struct)?;
        let mut off = self.off_struct;
        let mut depth = 0usize;

        loop {
            if off + 4 > struct_end {
                return None;
            }
            let token = self.u32_at(off)?;
            off += 4;

            match token {
                FDT_BEGIN_NODE => {
                    let name = self.cstr_struct(off)?;
                    off = off.checked_add(align4(name.len() + 1))?;
                    let after_name = off;

                    if let Some(value) = self.property(after_name, "compatible") {
                        for entry in value.split(|&b| b == 0) {
                            if entry == compatible.as_bytes() {
                                return Some(after_name);
                            }
                        }
                    }

                    depth += 1;
                }
                FDT_END_NODE => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                FDT_PROP => {
                    self.skip_prop(&mut off)?;
                }
                FDT_NOP => {}
                FDT_END => break,
                _ => return None,
            }
        }

        None
    }

    pub fn property(&self, node_after_name: usize, name: &str) -> Option<&'a [u8]> {
        let struct_end = self.off_struct.checked_add(self.size_struct)?;
        let mut off = node_after_name;

        loop {
            if off + 4 > struct_end {
                return None;
            }
            let token = self.u32_at(off)?;
            off += 4;

            match token {
                FDT_PROP => {
                    if off + 8 > struct_end {
                        return None;
                    }
                    let len = self.u32_at(off)? as usize;
                    off += 4;
                    let name_off = self.u32_at(off)? as usize;
                    off += 4;

                    if off + len > struct_end {
                        return None;
                    }
                    let value = &self.data[off..off + len];
                    let prop_name = self.cstr_strings(name_off)?;
                    off = off.checked_add(align4(len))?;

                    if prop_name == name.as_bytes() {
                        return Some(value);
                    }
                }
                FDT_NOP => {}
                _ => return None,
            }
        }
    }

    pub fn property_u32(&self, node_after_name: usize, name: &str) -> Option<u32> {
        let value = self.property(node_after_name, name)?;
        if value.len() < 4 {
            return None;
        }
        Some(u32::from_be_bytes(value[..4].try_into().ok()?))
    }

    pub fn property_u64(&self, node_after_name: usize, name: &str) -> Option<u64> {
        let value = self.property(node_after_name, name)?;
        if value.len() < 8 {
            return None;
        }
        Some(u64::from_be_bytes(value[..8].try_into().ok()?))
    }

    pub fn property_str(&self, node_after_name: usize, name: &str) -> Option<&'a str> {
        let value = self.property(node_after_name, name)?;
        let len = value.iter().position(|&b| b == 0).unwrap_or(value.len());
        core::str::from_utf8(&value[..len]).ok()
    }

    pub fn reg_addr(&self, node_after_name: usize) -> Option<u64> {
        let bytes = self.address_cells as usize * 4;
        if bytes == 0 || bytes > 8 {
            return None;
        }
        let value = self.property(node_after_name, "reg")?;
        if value.len() < bytes {
            return None;
        }
        Some(read_be_cells(&value[..bytes]))
    }

    pub fn reg_size(&self, node_after_name: usize) -> Option<u64> {
        let addr_bytes = self.address_cells as usize * 4;
        let size_bytes = self.size_cells as usize * 4;
        if size_bytes == 0 || size_bytes > 8 {
            return None;
        }
        let value = self.property(node_after_name, "reg")?;
        if value.len() < addr_bytes + size_bytes {
            return None;
        }
        Some(read_be_cells(&value[addr_bytes..addr_bytes + size_bytes]))
    }

    pub fn reg_pairs(&self, node_after_name: usize) -> ([(u64, u64); 8], usize) {
        let mut pairs = [(0u64, 0u64); 8];
        let mut count = 0usize;
        let addr_bytes = self.address_cells as usize * 4;
        let size_bytes = self.size_cells as usize * 4;
        let cell_bytes = addr_bytes + size_bytes;
        if cell_bytes == 0 || addr_bytes > 8 || size_bytes > 8 {
            return (pairs, 0);
        }

        let Some(value) = self.property(node_after_name, "reg") else {
            return (pairs, 0);
        };
        let mut offset = 0usize;
        while offset + cell_bytes <= value.len() && count < pairs.len() {
            pairs[count].0 = read_be_cells(&value[offset..offset + addr_bytes]);
            pairs[count].1 = read_be_cells(&value[offset + addr_bytes..offset + cell_bytes]);
            count += 1;
            offset += cell_bytes;
        }
        (pairs, count)
    }

    pub fn child_nodes(&self, parent_after_name: usize) -> ([usize; 16], usize) {
        let mut children = [0usize; 16];
        let mut count = 0usize;
        let struct_end = match self.off_struct.checked_add(self.size_struct) {
            Some(value) => value,
            None => return (children, 0),
        };
        let mut off = parent_after_name;
        let mut depth = 0usize;

        while off + 4 <= struct_end && count < children.len() {
            let Some(token) = self.u32_at(off) else {
                break;
            };
            off += 4;

            match token {
                1 => {
                    if depth == 0 {
                        let Some(name) = self.cstr_struct(off) else {
                            break;
                        };
                        let aligned = off + align4(name.len() + 1);
                        if aligned <= struct_end {
                            children[count] = aligned;
                            count += 1;
                        }
                        off = aligned;
                    } else {
                        let Some(name) = self.cstr_struct(off) else {
                            break;
                        };
                        off += align4(name.len() + 1);
                    }
                    depth += 1;
                }
                2 => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                3 => {
                    if !self.skip_prop(&mut off).is_some() {
                        break;
                    }
                }
                4 => {}
                9 => break,
                _ => break,
            }
        }

        (children, count)
    }
}

fn read_be_cells(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for &byte in bytes {
        value = (value << 8) | byte as u64;
    }
    value
}
