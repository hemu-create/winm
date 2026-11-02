# Phase 0: Research and Feasibility

Status: complete on 2026-11-02.

## Objective

Determine whether a bare-metal Windows 11 ARM64 boot on Apple Silicon is
technically plausible, identify the real blockers, and choose a first target
and boot architecture before writing production code.

## Questions

1. Does m1n1 support the target SoC and interrupt controller?
2. What ABI does m1n1 use to hand control to a payload?
3. Can an EL2 component provide the PSCI/GIC/timer model Windows expects?
4. What are the page-size and DMA constraints?
5. Is there existing UEFI/ACPI code we can reuse?
6. What is the least unsafe installation path?
7. What legal and licensing constraints apply?

## Findings

### Target hardware

- The first target is the 13-inch M3 MacBook Air, SoC `t8122`.
- M3 uses AICv3. The Linux binding is `apple,t8122-aic3`; AICv3 is used from M3
  through at least M5.
- AICv3 is similar to AICv2 but uses additional Apple Device Tree offsets:
  `aic-iack-offset`, `cap0-offset`, and `maxnumirq-offset`.
- m1n1 supports AIC1/2/3 and M3 (`t8122`), including the M3 MCC blocks.
- The user's "M3 is unsupported" premise is outdated: Asahi Linux officially
  supports M3 as of September 2026, and installer support was merged.

Sources:

- https://asahilinux.org/2026/09/m2-episode-1
- https://lkml.iu.edu/2602.2/09020.html
- https://asahilinux.org/docs/platform/quirks/

### m1n1 boot ABI

- m1n1 detects an ARM64 Linux Image by the `ARM\x64` magic at offset `0x38`.
- It copies the image to a 2 MiB-aligned heap block and calls its entry.
- `kboot_boot()` sets the next-stage entry and puts the generated FDT pointer in
  `args[0]`; `main.c` calls the entry with five arguments.
- b00t therefore receives the FDT in `x0`, not an Apple `boot_args` pointer.
- m1n1 calls `mmu_shutdown()` before the next stage, disabling the MMU, caches,
  and exceptions. b00t must set up its own translation regime, vectors, and
  stack.
- m1n1 remains at EL2 and does not drop to EL1 before the payload.
- The m1n1 proxy can set `next_stage.entry` and arguments via `P_CALL`, which is
  useful for a fast b00t development loop.

m1n1 files inspected:

- `src/payload.c`
- `src/kboot.c`
- `src/kboot.h`
- `src/xnuboot.h`
- `src/main.c`
- `src/chainload.c`
- `src/aic.c`
- `src/aic_regs.h`
- `src/memory.c`
- `src/exception.c`
- `rust/src/lib.rs`
- `rust/src/chainload.rs`

### Interrupts and timers

- Apple uses AIC, not a GIC.
- AIC timer/PMU sources are FIQ-capable.
- Windows ARM64 has inbox GIC support; Raspberry Pi 4 Windows prior art uses
  GICv2.
- A virtual GICv2 can be emulated at EL2 with stage-2 MMIO traps and
  `HCR_EL2.VI`. No GIC virtualization hardware is required.
- A GICv2 emulation is the lowest-risk first path. An AIC HAL extension is a
  later performance path and carries undocumented-API and signing risk.
- Windows HAL extensions are loaded via the ACPI CSRT table; the CSRT resource
  group ID selects the extension, and the descriptor format is defined by the
  extension writer.

Source:

- https://learn.microsoft.com/en-us/windows-hardware/drivers/bringup/acpi-system-description-tables

### Memory and DMA

- Windows ARM64 uses 4 KiB MMU pages. Wine 10.0 requires a 4 KiB system page
  size because of this.
- The 64 KiB figure often seen in Windows documentation is allocation
  granularity, not the MMU page size.
- Apple DART uses a 16 KiB granule.
- A 16 KiB NT kernel is not a viable path. Plan for 4 KiB CPU translation and
  16 KiB-aligned DMA/bounce buffers.
- Do not expose IORT until the DART/SMMU model is validated. A wrong IORT is
  worse than no IORT.

### UEFI and ACPI

- Upstream U-Boot has ARM ACPI writers for FADT/MADT/GTDT/IORT/MCFG in
  `lib/acpi/acpi_table.c`, but the Asahi `apple_m1_defconfig` does not enable
  ACPI or EFI_LOADER. U-Boot's UEFI is EBBR-focused, not a full Windows platform.
- EDK2's `ArmVirtPkg` is the closest reusable ARM platform package.
- Final design: a custom EDK2 Apple Silicon platform, derived from
  `ArmVirtPkg`/`ArmVirtQemu`, with b00t providing the platform handoff.
- U-Boot is a Phase 1 scaffold/loader experiment only. coreboot is rejected
  because it has no Apple Silicon port and would duplicate m1n1 initialization.
- b00t should generate ACPI tables from the FDT and pass them to EDK2; EDK2's
  ACPI table DXE publishes them.

### Security and ABI

- Windows ARM64 uses x18 for the TEB/KPCR. Apple reserves x18 and macOS clears
  it in software on context switch/exception return.
- Apple Silicon hardware supports preserving x18 on exception entry.
- macOS 26.5 adds an "x18 custom ABI" mode with a
  `com.apple.security.custom-x18-abi-toggle` entitlement. Older macOS versions
  did not clear x18; current macOS preserves compatibility for older SDKs.
- b00t must preserve x18 and TPIDR_EL1 across EL2 exceptions. This is testable
  and not a showstopper by itself.
- Windows Boot Manager requires UEFI variables. Apple has no native variable
  store. A file-backed variable store on the ESP is the Phase 1 approach.

### Installation

- Reuse asahi-installer's APFS resize flow (MIT) rather than reimplementing it.
- Use `kmutil configure-boot` for the custom boot object.
- Download/verify an official Windows ARM64 ISO or build from UUP locally.
- WIM apply with wimlib; NTFS with ntfs-3g/mkfs.ntfs.
- Offline driver injection is hard; a disposable Windows VM/WinPE with DISM and
  bcdboot is the practical Phase 2 path.
- Windows 11 Setup can be made compatible with
  `HKLM\SYSTEM\Setup\LabConfig` bypass values, but this must be applied to the
  boot.wim WinPE environment, not only install.wim.
- Test signing is a development feature, not a shipping security model.

## Decisions

- Target base M3 Air (`t8122`) first.
- b00t is an independent Rust `no_std` EL2 component, MIT licensed.
- b00t is packaged as an ARM64 Image payload and receives the FDT in `x0`.
- Final UEFI environment is EDK2, not U-Boot.
- Interrupt strategy starts with virtual GICv2 emulation.
- Windows uses 4 KiB pages; DART uses 16 KiB-aligned buffers/bounce buffers.
- No IORT in Phase 1.
- Test signing for development; production signing is a later phase.
- Installer reuses asahi-installer for APFS resizing with attribution.

## Artifacts

- `README.md`
- `docs/architecture.md`
- `docs/roadmap.md`
- `docs/hard-problems.md`
- `docs/drivers.md`
- `docs/debugging.md`
- `docs/risk-register.md`
- `docs/legal.md`
- `docs/decisions/`
- `b00t/` skeleton
- `installer/` skeleton
- `hal-aic/`, `drivers/nvme/`, `uefi/` stubs

## Exit criteria

- A documented boot chain from iBoot to Windows.
- A concrete EL2/EL1 split.
- Known blockers with experiments.
- A first target and a stop/go list.
- No claim that Windows already boots.

## Open questions carried into Phase 1

1. Exact AICv3 ack/EOI behavior on `t8122`.
2. Windows GICv2 register access pattern and latency tolerance.
3. DART domain assignment and 4K/16K DMA mapping details.
4. EDK2 platform port size and the minimum viable DXE set.
5. File-backed UEFI variable durability.
6. x18 preservation under real EL2 interrupt load.
7. Thermal/power limits without macOS.
