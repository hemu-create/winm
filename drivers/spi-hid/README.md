# SPI-HID Driver Notes

Apple keyboards and trackpads are SPI-HID devices with Apple-specific
descriptors and power sequences.

## Strategy

- Phase 2: use USB input.
- Phase 4: implement a Windows HID transport or dedicated input driver.
- Keep power sequencing conservative and reversible.

## Open questions

- Whether the inbox SPI controller driver can bind to Apple SPI.
- Exact HID descriptor and report format.
- Power and reset sequencing.
