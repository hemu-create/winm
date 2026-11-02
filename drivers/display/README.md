# Display Driver Notes

Phase 1-3 use the m1n1 simple framebuffer and EDK2 GOP. Windows uses the
Microsoft basic display driver.

## Later work

- DCP driver for native resolution and brightness.
- WDDM display-only driver.
- Multi-display and notch handling.
- GPU acceleration is a separate, much larger effort.

## Open questions

- DCP register and command model on M3.
- Brightness and backlight control.
- Display power and sleep integration.
