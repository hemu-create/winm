# ADR 0001: UEFI Environment

## Status

Accepted for Phase 1 planning; implementation pending.

## Context

WinM needs a UEFI environment that can load Windows Boot Manager and provide
ACPI, GOP, runtime services, and variables on Apple Silicon.

Options:

1. U-Boot's UEFI implementation on the Asahi U-Boot branch.
2. EDK2 with a custom Apple Silicon platform package.
3. coreboot plus a UEFI payload.
4. A custom UEFI implementation.

## Decision

Use EDK2 with a custom Apple Silicon platform package as the final UEFI
environment. Use U-Boot only as a Phase 1 loader/scaffolding experiment.

Start the EDK2 port from `ArmVirtPkg/ArmVirtQemu` and replace QEMU-specific
services with b00t handoff services.

## Rationale

- EDK2 has mature ACPI table publishing, variable services, BDS, GOP, and
  runtime-service infrastructure.
- U-Boot's Asahi defconfig does not enable ACPI/EFI_LOADER and is targeted at
  EBBR Linux boot, not a Windows platform.
- Upstream U-Boot does contain ARM ACPI writers, but integrating a complete
  Windows-capable platform there is more work than extending EDK2.
- coreboot has no Apple Silicon port and would duplicate m1n1 initialization.
- A custom UEFI implementation is unnecessary and increases security risk.

## Consequences

- We own an EDK2 platform package, including memory map, ACPI, GOP, variables,
  RTC/RNG/reset, and serial.
- b00t must provide a stable handoff ABI and HVC services.
- U-Boot remains useful for early image loading and as a comparison point, but
  is not the product path.
