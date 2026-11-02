pub const TABLE_HEADER_LEN: usize = 36;
pub const RSDP_LEN: usize = 36;

pub fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte))
}

pub fn init_table_header(
    table: &mut [u8],
    signature: &[u8; 4],
    length: u32,
    revision: u8,
    oem_id: &[u8; 6],
    oem_table_id: &[u8; 8],
) {
    let length = length as usize;
    if table.len() < TABLE_HEADER_LEN || length < TABLE_HEADER_LEN || length > table.len() {
        return;
    }

    table.fill(0);
    table[0..4].copy_from_slice(signature);
    table[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    table[8] = revision;
    table[9] = 0;
    table[10..16].copy_from_slice(oem_id);
    table[16..24].copy_from_slice(oem_table_id);
    table[24..28].copy_from_slice(&1u32.to_le_bytes());
    table[28..32].copy_from_slice(b"WINM");
    table[32..36].copy_from_slice(&1u32.to_le_bytes());

    let sum = checksum(&table[..length]);
    table[9] = 0u8.wrapping_sub(sum);
}

pub fn init_rsdp(rsdp: &mut [u8; RSDP_LEN], xsdt: u64) {
    rsdp.fill(0);
    rsdp[0..8].copy_from_slice(b"RSD PTR ");
    rsdp[9..15].copy_from_slice(b"WINM  ");
    rsdp[15] = 2;
    rsdp[20..24].copy_from_slice(&(RSDP_LEN as u32).to_le_bytes());
    rsdp[24..32].copy_from_slice(&xsdt.to_le_bytes());

    let sum = checksum(&rsdp[..20]);
    rsdp[8] = 0u8.wrapping_sub(sum);

    let extended_sum = checksum(&rsdp[..RSDP_LEN]);
    rsdp[32] = 0u8.wrapping_sub(extended_sum);
}

pub fn table_header_valid(table: &[u8]) -> bool {
    table.len() >= TABLE_HEADER_LEN && checksum(&table[..TABLE_HEADER_LEN]) == 0
}

pub fn rsdp_valid(rsdp: &[u8; RSDP_LEN]) -> bool {
    rsdp[0..8] == *b"RSD PTR " && checksum(&rsdp[..20]) == 0 && checksum(&rsdp[..RSDP_LEN]) == 0
}

pub fn build_xsdt(buffer: &mut [u8], entries: &[u64]) -> usize {
    let length = TABLE_HEADER_LEN + entries.len() * 8;
    if buffer.len() < length {
        return 0;
    }
    init_table_header(buffer, b"XSDT", length as u32, 1, b"WINM  ", b"WINM    ");
    for (index, entry) in entries.iter().enumerate() {
        let offset = TABLE_HEADER_LEN + index * 8;
        buffer[offset..offset + 8].copy_from_slice(&entry.to_le_bytes());
    }
    length
}

pub fn build_fadt(buffer: &mut [u8], dsdt: u64) -> usize {
    const FADT_LEN: usize = 276;
    if buffer.len() < FADT_LEN {
        return 0;
    }
    buffer.fill(0);
    init_table_header(buffer, b"FACP", FADT_LEN as u32, 6, b"WINM  ", b"WINM    ");

    // Minimal ARM-oriented FADT fields. The rest remain zero until the platform
    // model is completed and validated.
    buffer[36..40].copy_from_slice(&0u32.to_le_bytes()); // FirmwareCtrl
    buffer[40..44].copy_from_slice(&(dsdt as u32).to_le_bytes()); // Dsdt
    buffer[129] = 0x03; // ARM boot arch: PSCI compliant + HVC
    buffer[130] = 0x01; // minor version
    buffer[131] = 0x00; // errata

    // X_DSDT
    buffer[148..156].copy_from_slice(&dsdt.to_le_bytes());

    FADT_LEN
}

pub fn build_madt(buffer: &mut [u8]) -> usize {
    const MADT_LEN: usize = 44 + 80 + 24;
    if buffer.len() < MADT_LEN {
        return 0;
    }
    buffer.fill(0);
    init_table_header(buffer, b"APIC", MADT_LEN as u32, 5, b"WINM  ", b"WINM    ");

    // Local APIC address and flags.
    buffer[36..40].copy_from_slice(&0u32.to_le_bytes());
    buffer[40..44].copy_from_slice(&1u32.to_le_bytes());

    // GIC CPU interface subtable (type 11, length 80).
    buffer[44] = 11;
    buffer[45] = 80;
    buffer[44 + 8..44 + 12].copy_from_slice(&0u32.to_le_bytes()); // CPU interface number
    buffer[44 + 12..44 + 16].copy_from_slice(&0u32.to_le_bytes()); // ACPI processor UID
    buffer[44 + 16..44 + 20].copy_from_slice(&0u32.to_le_bytes()); // flags
    buffer[44 + 20..44 + 24].copy_from_slice(&0u32.to_le_bytes()); // parking protocol version
    buffer[44 + 24..44 + 32].copy_from_slice(&0u64.to_le_bytes()); // performance interrupt GSIV
    buffer[44 + 32..44 + 40].copy_from_slice(&0u64.to_le_bytes()); // parked address
    buffer[44 + 40..44 + 48].copy_from_slice(&0u64.to_le_bytes()); // physical base address
    buffer[44 + 48..44 + 52].copy_from_slice(&27u32.to_le_bytes()); // GICV
    buffer[44 + 52..44 + 56].copy_from_slice(&27u32.to_le_bytes()); // GICH
    buffer[44 + 56..44 + 60].copy_from_slice(&0u32.to_le_bytes()); // VGIC maintenance interrupt
    buffer[44 + 60..44 + 64].copy_from_slice(&0u32.to_le_bytes()); // GICR base address
    buffer[44 + 64..44 + 68].copy_from_slice(&0u32.to_le_bytes()); // MPIDR

    // GIC distributor subtable (type 12, length 24).
    let dist = 44 + 80;
    buffer[dist] = 12;
    buffer[dist + 1] = 24;
    buffer[dist + 2..dist + 4].copy_from_slice(&0u16.to_le_bytes()); // reserved
    buffer[dist + 4..dist + 8].copy_from_slice(&0u32.to_le_bytes()); // GICD physical base
    buffer[dist + 8..dist + 12].copy_from_slice(&0u32.to_le_bytes()); // system vector base
    buffer[dist + 12..dist + 16].copy_from_slice(&0u32.to_le_bytes()); // GIC version

    MADT_LEN
}

pub fn build_gtdt(buffer: &mut [u8]) -> usize {
    const GTDT_LEN: usize = 96;
    if buffer.len() < GTDT_LEN {
        return 0;
    }
    buffer.fill(0);
    init_table_header(buffer, b"GTDT", GTDT_LEN as u32, 2, b"WINM  ", b"WINM    ");

    // CntControlBase physical address and flags.
    buffer[36..44].copy_from_slice(&0u64.to_le_bytes());
    buffer[44..48].copy_from_slice(&0u32.to_le_bytes());
    buffer[48..52].copy_from_slice(&0u32.to_le_bytes()); // secure PL1 timer GSIV
    buffer[52..56].copy_from_slice(&0u32.to_le_bytes());
    buffer[56..60].copy_from_slice(&27u32.to_le_bytes()); // non-secure PL1 timer GSIV
    buffer[60..64].copy_from_slice(&0u32.to_le_bytes());
    buffer[64..68].copy_from_slice(&27u32.to_le_bytes()); // virtual timer GSIV
    buffer[68..72].copy_from_slice(&0u32.to_le_bytes());
    buffer[72..76].copy_from_slice(&27u32.to_le_bytes()); // hypervisor timer GSIV
    buffer[76..80].copy_from_slice(&0u32.to_le_bytes());
    buffer[80..84].copy_from_slice(&0u32.to_le_bytes()); // virtual PL2 timer GSIV
    buffer[84..88].copy_from_slice(&0u32.to_le_bytes());

    GTDT_LEN
}

pub fn build_spcr(buffer: &mut [u8], uart_base: u64) -> usize {
    const SPCR_LEN: usize = 80;
    if buffer.len() < SPCR_LEN {
        return 0;
    }
    buffer.fill(0);
    init_table_header(buffer, b"SPCR", SPCR_LEN as u32, 2, b"WINM  ", b"WINM    ");

    buffer[36] = 0; // interface type: 16550
    buffer[37] = 3; // reserved
    buffer[40..44].copy_from_slice(&0u32.to_le_bytes()); // interrupt type
    buffer[44..48].copy_from_slice(&27u32.to_le_bytes()); // IRQ
    buffer[48..56].copy_from_slice(&uart_base.to_le_bytes()); // base address
    buffer[56..57].copy_from_slice(&[115200u32 as u8]); // baud rate (encoded)
    buffer[58] = 8; // terminal type: ANSI
    buffer[59] = 0; // language
    buffer[60] = 0; // PCI device ID
    buffer[61] = 0; // PCI vendor ID
    buffer[64..72].copy_from_slice(&0u64.to_le_bytes()); // PCI bar
    buffer[72] = 0; // PCI device number
    buffer[73] = 0; // PCI function number
    buffer[74] = 0; // PCI segment
    buffer[76] = 0; // reserved

    SPCR_LEN
}

pub fn build_dbg2(buffer: &mut [u8], uart_base: u64) -> usize {
    const DBG2_LEN: usize = 128;
    if buffer.len() < DBG2_LEN {
        return 0;
    }
    buffer.fill(0);
    init_table_header(buffer, b"DBG2", DBG2_LEN as u32, 0, b"WINM  ", b"WINM    ");

    // Offset to first debug device information structure.
    buffer[36..40].copy_from_slice(&48u32.to_le_bytes());
    buffer[40..44].copy_from_slice(&1u32.to_le_bytes()); // number of debug devices

    let dev = 48usize;
    buffer[dev..dev + 2].copy_from_slice(&0x8000u16.to_le_bytes()); // device type: 16550
    buffer[dev + 2..dev + 4].copy_from_slice(&0u16.to_le_bytes()); // device subtype
    buffer[dev + 4..dev + 6].copy_from_slice(&1u16.to_le_bytes()); // register count
    buffer[dev + 6..dev + 8].copy_from_slice(&0u16.to_le_bytes()); // namepath length
    buffer[dev + 8..dev + 12].copy_from_slice(&64u32.to_le_bytes()); // namepath offset
    buffer[dev + 12..dev + 16].copy_from_slice(&0u32.to_le_bytes()); // OEM data length
    buffer[dev + 16..dev + 20].copy_from_slice(&0u32.to_le_bytes()); // OEM data offset
    buffer[dev + 20..dev + 24].copy_from_slice(&4u32.to_le_bytes()); // port type
    buffer[dev + 24..dev + 28].copy_from_slice(&0u32.to_le_bytes()); // port subtype
    buffer[dev + 28..dev + 32].copy_from_slice(&0u32.to_le_bytes()); // reserved
    buffer[dev + 32..dev + 40].copy_from_slice(&uart_base.to_le_bytes()); // base address
    buffer[dev + 40..dev + 44].copy_from_slice(&0u32.to_le_bytes()); // register stride
    buffer[dev + 44..dev + 48].copy_from_slice(&1u32.to_le_bytes()); // address size

    DBG2_LEN
}
