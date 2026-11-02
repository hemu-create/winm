# USB Driver Notes

Phase 2 uses USB for keyboard and install media. The target uses DWC3 behind
Apple-specific PHY, mux, and Type-C PD controllers.

## Strategy

- Prefer the inbox Windows xHCI driver if ACPI can describe the controller.
- Describe DWC3, PHY, and Type-C resources with `_CRS` and `_DSD`.
- Use a small platform driver or ACPI methods for role/alt-mode if needed.
- Provide USB Ethernet as an interim network/debug path.

## Open questions

- Whether the inbox xHCI driver binds to Apple DWC3 without a custom driver.
- Type-C PD/alt-mode sequencing.
- Interrupt routing from AIC to the emulated GIC.
