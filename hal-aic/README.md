# AIC HAL Extension Research

Status: research stub. Not part of Phase 1.

## What a HAL extension is

Windows HAL extensions are SoC-specific DLLs that adapt the Windows hardware
abstraction layer to non-standard core silicon. Microsoft documents that they
are identified and loaded through the ACPI CSRT table:

- CSRT resource groups identify a hardware type.
- A resource group ID selects the HAL extension.
- Resource descriptor formats are defined by the HAL extension writer.

Source: https://learn.microsoft.com/en-us/windows-hardware/drivers/bringup/acpi-system-description-tables

## Why it might be useful

The Phase 1 design emulates a GICv2 at EL2. A native AIC HAL extension could
remove that emulation and reduce interrupt latency.

## Why it is high risk

- The exact HAL extension interface for interrupt controllers is not publicly
  documented.
- CSRT resource-group descriptors are extension-defined, so we would need
  Windows-facing validation.
- Kernel/HAL extensions require signing and a supported distribution path.
- A wrong implementation can bugcheck or corrupt the system early in boot.
- Windows may not accept a non-GIC interrupt controller through this path even
  if the extension loads.

## Decision

Do not depend on a HAL extension for Phase 1. Keep the virtual GICv2 path as the
primary implementation and treat this directory as a place for experiments,
notes, and a WDK skeleton once the interface is understood.

## Minimal experiment

1. Build a signed test HAL extension that only logs its load and resource
   groups.
2. Add a CSRT table with a vendor-defined resource group.
3. Verify Windows loads the extension and passes the expected descriptors.
4. Only then attempt to register an interrupt-controller interface.

## Files

- `include/winm_aic.h`: conceptual interface, not a Windows ABI.
