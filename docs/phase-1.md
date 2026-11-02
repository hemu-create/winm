# Phase 1: b00t and Minimal UEFI

Status: implementation starting.

## Goal

Boot a minimal EDK2-based UEFI environment from b00t on the target M3 MacBook
Air, with serial and framebuffer output, working ACPI tables, timer interrupts,
PSCI CPU bring-up, and persistent UEFI variables.

Phase 1 does not install Windows. It proves that the platform model is credible
enough to attempt Windows PE.

## Deliverables

1. `b00t.img`: ARM64 Image-compatible payload accepted by m1n1.
2. FDT discovery and a platform summary over UART.
3. EL2 MMU/stage-2 translation and exception vectors.
4. AIC initialization and physical interrupt handling.
5. Virtual GICv2 distributor/CPU interface.
6. PSCI 1.1 over HVC.
7. ACPI table handoff.
8. EDK2 Apple Silicon platform package that reaches the UEFI shell.
9. GOP using the m1n1 simple framebuffer.
10. File-backed UEFI variable store.
11. Recovery and test harness.
12. Debug logging with stable message IDs.

## b00t boot contract

Input:

- `x0` = physical pointer to the m1n1-generated FDT.
- EL2, MMU off, caches off, interrupts masked.
- Image loaded at a 2 MiB-aligned address.

Output:

- b00t remains resident at EL2.
- EL1 entry point and handoff structure are passed to EDK2.
- Physical IRQ/FIQ are owned by b00t.
- ACPI tables and memory map are available to EDK2.

b00t must not return to m1n1. If it cannot continue, it prints a diagnostic and
parks the CPU.

## b00t image layout

```text
0x0000  ARM64 Image header (branch, text_offset, image_size, flags, magic)
0x0040  EL2 entry code
        .text
        .rodata
        .data
        .bss (not copied; zeroed by entry code)
        stack
```

The image must be position-independent until a self-relocator runs. Avoid
absolute function pointers and static pointers in the early image. If the build
introduces `R_AARCH64_RELATIVE` relocations, process `.rela.dyn` before entering
Rust.

## Memory reservations

b00t must reserve, at minimum:

- Its own image and `.bss`.
- EL2 stacks and vectors.
- EL2 stage-1 and stage-2 page tables.
- ACPI blobs and handoff blob.
- EDK2 image, heap, and runtime buffers.
- Framebuffer.
- UART and AIC MMIO (device mappings).
- Any DART tables or bounce buffers.
- m1n1/FDT and reserved-memory regions from the FDT.

Everything else may be offered to UEFI as conventional memory, subject to the
FDT's reserved-memory list and actual hardware carveouts.

## ACPI tables

Phase 1 tables:

| Table | Purpose |
| --- | --- |
| RSDP | Root pointer |
| FADT | PSCI/HVC, reset, PM timer |
| MADT | Emulated GICv2, CPUs, timer PPIs |
| GTDT | Generic timer |
| DBG2 | Debug UART |
| SPCR | Serial console |
| MCFG | PCIe ECAM, if PCIe is described |
| DSDT | Platform devices and methods |
| SSDT | CPU/device supplements |

Rules:

- Build AML with `iasl` at build time; do not hand-write AML.
- Validate every table with a checksum and an AML parser.
- Do not advertise IORT until DART is proven.
- Do not advertise CSRT unless a HAL extension is present.
- Keep table generation deterministic and reproducible.

## EDK2 platform

Start from `ArmVirtPkg/ArmVirtQemu.dsc` and replace:

- fw_cfg with b00t handoff.
- QEMU-specific GIC/timer/CPU descriptions with b00t ACPI.
- QEMU variable store with a file-backed store on the ESP.
- QEMU RTC/reset with b00t HVC services.
- QEMU GOP with a simple framebuffer GOP.

Keep:

- `AcpiTableDxe` and ACPI platform DXE.
- Variable runtime DXE.
- UEFI shell, BDS, and boot manager.
- `SerialPortLib` adapted to the Apple UART.

Minimum boot target: UEFI shell over UART and framebuffer, with `acpidump`,
`memmap`, `drivers`, and `exit` working.

## Interrupt implementation

- Claim physical IRQ/FIQ at EL2.
- Initialize AIC with the ADT/FDT-provided offsets.
- Mask all sources during setup.
- Implement virtual GICv2 distributor and CPU interface.
- Map GIC INTIDs to AIC hardware IRQs.
- Inject virtual interrupts with `HCR_EL2.VI`.
- Implement timer PPI 27/30 and PMU PPI.
- Implement IPIs through AIC software interrupts.
- Test with a bare-metal EL1 timer/IRQ payload before EDK2.

## PSCI implementation

- PSCI version 1.1, HVC conduit.
- `PSCI_VERSION`, `CPU_ON`, `CPU_OFF`, `AFFINITY_INFO`, `SYSTEM_OFF`,
  `SYSTEM_RESET`, `MIGRATE_INFO_TYPE`.
- CPU_ON uses Apple's release mechanism; secondary CPUs enter EL1 at the
  requested entry point with the requested context ID.
- Track CPU state and reject invalid MPIDRs.
- Test with all performance cores and at least one efficiency core.

## UEFI variables

- File-backed store on the ESP, e.g. `\EFI\WinM\vars.fv`.
- Keep a backup copy and a generation counter.
- Flush on SetVariable and at ExitBootServices.
- Recover from a truncated/corrupt file by falling back to the backup.
- Verify Boot####, BootOrder, and BootNext survive reboot and power loss.

## Build and test loop

Development:

1. Build `b00t.img`.
2. Load via m1n1 proxy or m1n1 payload chain.
3. Capture UART log.
4. Run the EL1 test payload.
5. Build EDK2 and boot to shell.
6. Run ACPI and memory-map checks.

Production-like:

1. Install m1n1 stage 1 as a custom boot object on a sacrificial Mac.
2. Put b00t + EDK2 on the ESP.
3. Boot with the m1n1 payload chain.
4. Verify recovery by removing the boot object.

## Test plan

| Test | Pass condition |
| --- | --- |
| b00t entry | Prints FDT pointer/size and SoC type |
| FDT parse | Memory/reserved/UART/AIC/framebuffer match m1n1 log |
| MMU | Caches enabled; device MMIO works; no translation faults |
| Exceptions | Known exception prints ESR/FAR/ELR and recovers or parks |
| AIC | No spurious interrupts; mask/unmask works |
| Timer | 10 minutes of ticks without drift/hang |
| GICv2 | EL1 test payload receives timer IRQ and EOIs it |
| PSCI | All CPUs online; CPU_ON/OFF/AFFINITY_INFO correct |
| EDK2 | Shell prompt over UART and framebuffer |
| ACPI | `iasl -d` accepts tables; Linux `acpidump` parses them |
| Variables | Boot entries survive reboot and power loss |
| Recovery | Boot object removed; macOS boots |

## Exit criteria

- EDK2 shell stable for 30 minutes.
- ACPI tables valid and internally consistent.
- At least two CPUs online.
- Timer interrupts stable.
- No memory corruption in a 1 GiB memory test.
- UEFI variables persistent.
- Recovery documented and tested.
- A written go/no-go decision for Phase 2.

## Explicit non-goals

- Windows installation.
- NVMe/USB/display drivers for Windows.
- Secure Boot production keys.
- GPU acceleration.
- Sleep/thermal production policy.
- M3 Pro/Max support.
