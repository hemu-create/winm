# AIC Windows Driver Research

Status: research stub.

## Source

- m1n1: `src/aic.c`, `src/aic_regs.h`
- Linux/Asahi: AIC interrupt controller driver

## Windows strategy

Phase 1 uses b00t's virtual GICv2 emulation. A native AIC HAL extension is a
later performance path and requires undocumented HAL interfaces and signing.

## Skeleton goals

- Log AIC version and offsets.
- Acknowledge and complete interrupts.
- Mask/unmask sources.
- Map AIC events to GIC INTIDs in b00t.

## Open questions

- Exact HAL extension interrupt-controller interface.
- CSRT resource group descriptor format.
- Signing and distribution path.
