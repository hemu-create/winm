# DCP Display Driver Notes

Status: research stub.

## Source

- Asahi Linux DCP driver
- m1n1 display initialization and framebuffer setup

## Strategy

- Phase 1-3 use the m1n1 simple framebuffer and EDK2 GOP.
- Phase 4 implements DCP display and a WDDM display-only driver.
- GPU acceleration is separate and much larger.

## Open questions

- DCP register and command model on M3.
- Brightness and backlight control.
- Display power and sleep integration.
- Notch and multi-display handling.
