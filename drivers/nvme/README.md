# Apple NVMe Windows Driver

Status: design stub.

## Goal

Provide Windows with a usable internal NVMe namespace on Apple Silicon, either
through a native storport miniport or, for early bring-up, a b00t-provided block
service.

## Constraints

- Apple NVMe is behind DART.
- DART uses a 16 KiB granule.
- Windows uses 4 KiB pages.
- PRP lists and DMA buffers must respect the DART granule.
- The shared `drivers/common/winm_dma.*` helpers provide 16 KiB-aligned
  allocations and detect when a Windows 4 KiB buffer needs a bounce buffer.
- The controller's interrupts are delivered through AIC, which Windows sees as
  an emulated GICv2 in Phase 1.
- Reset, power, and queue teardown must not leave the DART domain in an
  inconsistent state.

## Phase 2A: b00t block service

- b00t owns the NVMe controller and DART domain.
- Windows uses a simple test miniport that forwards read/write/flush requests
  over HVC.
- This proves the Windows storage stack and BCD/installation flow without
  solving native DMA.
- Performance is not the goal.

## Phase 2B: native storport miniport

- Windows owns the NVMe queue pair.
- b00t grants a preconfigured DART domain or exposes a safe register window.
- The miniport allocates 16 KiB-aligned queues, PRP lists, and bounce buffers.
- Start with polling; add interrupts after the queue is stable.
- Validate surprise reset, large I/O, and pagefile/hibernation.

## Bring-up checklist

1. Identify controller, namespace, and doorbell layout from the FDT/ADT.
2. Map controller registers without conflicting with b00t.
3. Program admin queue and identify controller/namespace.
4. Read/write a scratch LBA with a 16 KiB-aligned buffer.
5. Add interrupt completion.
6. Add storport SRB handling.
7. Add flush, reset, and power transitions.
8. Stress with mixed 4K/16K I/O.

## Open questions

- Which DART domain does the NVMe controller use on `t8122`?
- Can Windows storport tolerate 16 KiB-aligned buffers for all transfers?
- How should b00t revoke or reset a domain on driver unload?
- Does the controller need Apple-specific power/clock sequencing after m1n1?
