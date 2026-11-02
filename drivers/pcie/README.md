# PCIe Driver Notes

Status: research stub.

## Source

- Asahi Linux PCIe host bridge driver
- m1n1 PCIe initialization

## Strategy

- Describe Apple PCIe host bridges in ACPI with correct `_CRS`, `_DSD`, and
  power resources.
- Prefer inbox Windows PCIe drivers where possible.
- Keep DART ownership in b00t initially.

## Open questions

- PCIe topology on M3.
- Power and reset sequencing.
- DART domain assignment.
- Interrupt routing to the emulated GIC.
