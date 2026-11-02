# ADR 0002: Interrupt Strategy

## Status

Accepted for Phase 1.

## Context

Apple uses AIC. Windows ARM64 expects a GIC. Options:

1. Emulate GICv2 at EL2.
2. Emulate GICv3 at EL2.
3. Write a Windows AIC HAL extension.
4. Expose a custom interrupt-controller driver.

## Decision

Start with a virtual GICv2 distributor and CPU interface emulated at EL2.
Keep an AIC HAL extension as a later performance path. Do not expose an
undocumented HAL extension in Phase 1.

## Design

- b00t runs at EL2 and owns physical IRQ/FIQ (`HCR_EL2.IMO/FMO`).
- AIC acknowledges/completes physical interrupts at EL2.
- Stage-2 translation traps guest accesses to the virtual GIC MMIO window.
- The virtual GIC implements:
  - Distributor: `CTLR`, `ISENABLER`, `ICENABLER`, `ISPENDR`, `ICPENDR`,
    `IPRIORITYR`, `ITARGETSR`, `ICFGR`, `SGIR`.
  - CPU interface: `CTLR`, `PMR`, `BPR`, `IAR`, `EOIR`, `RPR`, `HPPIR`,
    `DIR`.
- Physical AIC hwirqs map to GIC INTIDs.
- Timer PPIs 27/30 and PMU PPIs are synthesized.
- IPIs use AIC software interrupts.
- Virtual interrupts are injected with `HCR_EL2.VI`.

## Rationale

- Windows has inbox GIC support; Raspberry Pi 4 Windows prior art uses GICv2.
- GICv2 is simpler than GICv3 and does not require system-register trapping or
  GIC virtualization hardware.
- It avoids undocumented Windows HAL interfaces and signing requirements for
  Phase 1.
- Latency is acceptable for boot and early operation; native AIC performance can
  come later.

## Consequences

- b00t carries an interrupt-controller emulator and must be correct about EOI
  ordering.
- High interrupt rates (NVMe, USB) may expose latency; measure before optimizing.
- A HAL extension remains a research branch, not a dependency.
