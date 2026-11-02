# ADR 0005: ACPI Ownership

## Status

Accepted for Phase 1.

## Context

Windows needs ACPI tables describing CPUs, GIC, timers, debug UART, PCIe, and
platform devices. Apple provides a Device Tree, not ACPI.

## Decision

b00t owns platform discovery and ACPI table generation. It passes a handoff blob
with table addresses and sizes to EDK2. EDK2's ACPI platform DXE installs the
tables and publishes the RSDP.

AML is compiled from WinM ASL sources with `iasl` at build time and copied into
the blob; it is not hand-written at runtime.

## Rationale

- b00t already parses the FDT and knows the Apple-specific topology.
- EDK2 has mature ACPI table installation and validation infrastructure.
- Keeping generation in b00t avoids duplicating Apple platform knowledge in
  EDK2.
- Build-time AML is auditable and less error-prone than runtime string
  construction.

## Consequences

- b00t must produce internally consistent tables before entering EDK2.
- Table changes require rebuilds of both b00t and the ASL blobs.
- IORT/CSRT are omitted until their underlying models are proven.
- ACPI table validation is a Phase 1 exit criterion.
