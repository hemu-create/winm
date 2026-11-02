# ADR 0003: Page Size and DART

## Status

Accepted for Phase 1.

## Context

Windows ARM64 uses 4 KiB MMU pages. Apple DART uses a 16 KiB granule. A naive
4 KiB mapping of arbitrary Windows buffers into a DART domain will corrupt
memory or trigger faults.

## Decision

- Use 4 KiB translation for the EL2 stage-2, EDK2, and Windows environment.
- Treat DART as a separate 16 KiB-granule IOMMU.
- Require 16 KiB-aligned DMA buffers or bounce buffers.
- Do not expose an IORT/SMMU description in Phase 1.
- Keep DART domains owned by b00t initially; grant a Windows miniport a
  preconfigured domain only after the model is validated.

## Rationale

- Running NT with a 16 KiB translation granule is not viable.
- A wrong IORT can cause Windows to program the IOMMU incorrectly and corrupt
  memory.
- A b00t-owned DART domain and a narrow block service provide a safe bring-up
  path while the native miniport is developed.

## Consequences

- Storage performance may be lower until native DMA is enabled.
- The NVMe miniport must understand 4K/16K alignment and PRP list constraints.
- Any driver doing DMA must document its alignment requirements.
- IORT support is a separate, evidence-driven milestone.
