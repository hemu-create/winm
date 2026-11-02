# Architecture

## Goals

WinM boots Windows 11 ARM64 bare metal on Apple Silicon without a virtual
machine. It must preserve enough of the Apple platform model for Windows to
boot and install, while presenting the standard ARM64 UEFI/ACPI interfaces that
Windows expects.

The first target is the base 13-inch M3 MacBook Air (`t8122`, AICv3). M1/M2 and
larger M3 variants are follow-on targets, not Phase 1 requirements.

## Why a resident EL2 component is required

Windows and UEFI expect:

- PSCI CPU bring-up, although Apple has no EL3-provided PSCI.
- A GIC interrupt controller, although Apple uses the Apple Interrupt
  Controller (AIC).
- A 4 KiB MMU granule, while Apple DART uses a 16 KiB granule.
- ACPI tables and variable services, while Apple provides a Device Tree and no
  native UEFI firmware.
- A platform timer and interrupt routing compatible with the ARM generic
  timer and GIC model.

A small EL2 component, `b00t`, owns these Apple-specific details. It runs
below the EDK2/Windows EL1 environment, keeps the machine's physical interrupt
routing under control, and exposes a narrow HVC ABI for services that cannot be
provided by ordinary device emulation.

## Boot flow

```text
iBoot
  -> m1n1 stage 1
  -> m1n1 stage 2
       - initializes Apple SoC, PCIe, USB, display, NVMe, AIC
       - builds a Linux FDT from the Apple Device Tree
       - disables its MMU and exceptions
       - enters b00t at EL2 with x0 = FDT
  -> b00t
       - self-relocates or uses a position-independent image
       - builds EL2 stage-1 and stage-2 translation tables
       - installs EL2 vectors and claims physical IRQ/FIQ
       - parses FDT, memory, reserved-memory, AIC, UART, framebuffer
       - builds ACPI tables and a UEFI handoff blob
       - loads the EDK2-based UEFI image at EL1
  -> EDK2 UEFI
       - publishes ACPI, GOP, variable services, runtime services
       - loads \EFI\Microsoft\Boot\bootmgfw.efi
  -> Windows Boot Manager
  -> winload.efi
  -> ntoskrnl.exe
```

m1n1's payload ABI is documented in
`docs/decisions/0004-m1n1-payload-abi.md`. The important detail is that the
next stage receives the FDT pointer in `x0`, not an Apple `boot_args` pointer.

## Execution levels and ownership

| Level | Owner | Responsibilities |
| --- | --- | --- |
| EL2 | b00t | Stage-2 translation, physical IRQ/FIQ, AIC, timer routing, PSCI, DART ownership, RTC/RNG/reset, HVC ABI, watchdog |
| EL1 | EDK2 / Windows | UEFI services, ACPI interpretation, Windows kernel and drivers |
| EL0 | User mode | Windows applications |

b00t must not rely on m1n1 after entry. m1n1 disables its MMU, caches, and
exceptions before jumping. b00t therefore starts with an identity-mapped or
MMU-off environment and must bring up its own stack, vectors, and translation
regime.

## b00t components

### Early entry

- ARM64 Linux Image-compatible header at offset 0.
- Position-independent entry code; no absolute static pointers unless a
  self-relocator processes them.
- Stack in `.bss`, zeroed before Rust entry.
- UART based on the FDT `serial0` alias or `/chosen/stdout-path`.
- Panic path that prints to UART and parks the CPU.

### Platform discovery

- FDT parser with bounds-checked big-endian access.
- `/memory` and `/reserved-memory` parsing.
- `/chosen/framebuffer` parsing for simplefb/GOP.
- `/arm-io/aic` parsing, including AIC version and ADT-derived offsets.
- `/aliases/serial0` for the debug UART.
- Apple SoC identification (`t8122` first).

### Memory

- EL2 stage-1 page tables for b00t and device MMIO.
- EL2 stage-2 page tables for the EL1 UEFI/Windows environment.
- 4 KiB stage-2 pages for Windows compatibility.
- Explicit reservation of b00t, FDT, ACPI, page tables, framebuffer, and
  firmware-owned buffers.
- DART domains are owned by b00t or by a Windows driver; no IORT is exposed in
  Phase 1. DMA buffers crossing a DART boundary must be 16 KiB aligned or
  bounce-buffered.

### Interrupts

- Physical IRQ/FIQ routed to EL2 with `HCR_EL2.IMO/FMO`.
- AIC acknowledged and completed at EL2.
- Virtual GICv2 distributor and CPU interface emulated through stage-2 MMIO
  traps.
- Physical interrupts translated to GIC INTIDs and injected with
  `HCR_EL2.VI`.
- Timer and PMU PPIs synthesized from Apple timer/PMU events.
- IPIs sent through AIC software interrupts.
- See `docs/decisions/0002-interrupt-strategy.md`.

### CPU bring-up

- b00t implements PSCI 1.1 at EL2 over HVC.
- `CPU_ON` uses the Apple release-register/spin-table mechanism.
- `AFFINITY_INFO`, `SYSTEM_OFF`, `SYSTEM_RESET`, and `PSCI_VERSION` are
  implemented.
- The FADT advertises PSCI with the HVC conduit.
- Secondary CPUs enter EL1 with the expected PSCI context.

### ACPI

b00t builds or supplies:

- `RSDP`, `FADT`, `MADT`, `GTDT`, `DBG2`, `SPCR`, `MCFG`, `DSDT`, `SSDT`.
- `IORT` only after the DART/SMMU model is validated; initially omitted so
  Windows uses identity DMA with driver-managed DART.
- `CSRT` only when a HAL extension path is enabled.

AML should be built at build time with `iasl` from WinM ASL sources, then copied
into the handoff blob by b00t. Hand-written AML is too error-prone.

### UEFI handoff

b00t loads an EDK2-based firmware image and passes:

- Memory map and reserved ranges.
- ACPI table addresses and sizes.
- Framebuffer information.
- UART information.
- HVC service ABI version and function IDs.
- Boot policy (macOS, Windows, recovery, test signing).

The EDK2 platform package installs the ACPI tables, exposes GOP, provides
file-backed UEFI variables, and forwards RTC/RNG/reset/variable persistence to
b00t over HVC.

### Runtime services

- Variable store: file-backed on the EFI System Partition during Phase 1;
  a dedicated reserved region may replace it later.
- RTC: Apple PMU/SMC-backed.
- RNG: Apple hardware RNG.
- Reset/shutdown: PSCI HVC to b00t.
- Capsule update: out of scope initially.
- Secure Boot: out of scope for Phase 1; test signing first.

## Memory map strategy

The exact addresses must come from the FDT and hardware probing, not from
guesses. The general layout is:

```text
low DRAM
  b00t image and data
  EL2 stacks and vectors
  EL2 stage-1 tables
  EL2 stage-2 tables
  ACPI blobs and handoff blob
  EDK2 firmware image and heap
  UEFI runtime/variable buffers
  framebuffer carveout (from FDT)
  Windows RAM
high DRAM
  reserved firmware/SEP/display regions
```

All reserved regions must be removed from the UEFI memory map and from the
Windows `EfiConventionalMemory` view. The framebuffer must be marked as
`EfiMemoryMappedIO` or a reserved runtime region, not as free RAM.

## Interrupt path

```text
AIC FIQ/IRQ
  -> EL2 vector
  -> b00t AIC handler
  -> translate hwirq to GIC INTID
  -> update virtual GIC state
  -> HCR_EL2.VI
  -> EL1 GIC IAR/EOIR emulation
  -> Windows ISR
```

Edge-triggered AIC sources may require software retriggering. Level-triggered
sources are masked/unmasked at the AIC as the virtual GIC state changes.
EOI ordering must preserve the AIC's ack/complete semantics.

## Storage and DMA

- NVMe is the primary Windows storage path.
- The Apple NVMe controller is behind DART; b00t either keeps the controller
  in a b00t-owned DART domain or grants a Windows driver ownership of a domain.
- Windows NT pages are 4 KiB. DART pages are 16 KiB. A storport miniport must
  use 16 KiB-aligned PRP lists and data buffers, or a bounce buffer.
- No IORT is exposed until the DART model is proven; exposing an incorrect
  IORT causes Windows to program the IOMMU incorrectly.

## Display

Phase 1 uses the m1n1 simple framebuffer: b00t reads `/chosen/framebuffer`,
marks it reserved, and EDK2 publishes a GOP that blits into it. Windows uses
the Microsoft basic display driver until a DCP/WDDM driver exists.

## Development and production modes

Development:

- b00t loaded directly by m1n1 proxy `P_CALL` for fast iteration.
- UART proxy and m1n1 hypervisor for early bring-up.
- EDK2 debug build with serial console.
- Windows PE from a USB or NVMe test partition.

Production:

- m1n1 stage 1 as the custom boot object.
- b00t as an ARM64 Image payload in the m1n1 payload chain.
- Signed/certified installer flow.
- Recovery path that can remove the boot object and restore the macOS boot
  policy.

## Open architecture questions

1. Exact AICv3 EOI/ack semantics on `t8122` under heavy interrupt load.
2. Whether Windows 11's GIC driver accepts a fully emulated GICv2 with no
   hardware GIC virtualization support.
3. Whether a HAL extension can replace the GIC path cleanly enough to justify
   the signing/undocumented-API risk.
4. DART domain assignment for NVMe, PCIe, USB, and display.
5. EDK2 runtime variable persistence and Windows Boot Manager behavior.
6. x18 preservation across EL2 exceptions on M3.
7. Thermal/power management without macOS's SMC policy.
