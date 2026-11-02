# Driver Plan

Windows support is not only a bootloader problem. The boot path needs storage,
input, display, and interrupt-controller support; daily use needs much more.
This document orders the work by dependency and risk.

## Priority definitions

- P0: required to boot/install or to debug the first Windows boot.
- P1: required for a usable desktop and normal installation media.
- P2: quality-of-life, performance, or later phases.

## P0

### AIC interrupt path

Two possible implementations:

1. EL2 virtual GICv2 emulation (Phase 1 default).
2. Windows HAL extension that drives AIC directly (later, performance path).

The first path needs no Windows driver at all; it is firmware emulation. The
second path needs a signed HAL extension and undocumented interface work. Start
with the emulation and keep the HAL extension as a research branch.

### Apple NVMe storport miniport

- Windows class: storage miniport (`storport.sys`).
- Hardware: Apple NVMe controller behind DART.
- Key constraints: 4 KiB NT pages, 16 KiB DART pages, PRP list alignment,
  queue depth, admin commands, reset, power.
- Phase 2 approach: b00t owns the DART domain, exposes a narrow block service or
  grants the miniport a preconfigured domain. Start with a simple polling
  miniport, then move to interrupts.
- Verification: `diskspd`/`fio` under Windows PE, surprise reset, large I/O,
  hibernation/pagefile stress.

### USB xHCI / DWC3 / Type-C

- Windows has an inbox xHCI driver; the question is whether the Apple DWC3
  controller and its PHY/PD/mux path can be described well enough in ACPI for
  the inbox driver.
- Type-C role/alt-mode and charger detection may require a custom driver or ACPI
  methods backed by b00t.
- Phase 2 needs USB keyboard and install media; a USB Ethernet adapter is a
  useful debug path.

### Display / GOP

- Use the m1n1 simple framebuffer for Phase 1/2.
- EDK2 GOP blits into the reserved framebuffer.
- Windows uses the Microsoft basic display driver.
- Do not attempt DCP/WDDM acceleration until installation is stable.

### Serial debug

- Apple UART via the FDT `serial0` alias.
- b00t, EDK2, and Windows debug output must share a documented policy.
- Windows kernel debugging can use serial, USB, or kdnet once a NIC works.

## P1

### SPI-HID keyboard/trackpad

- Apple keyboards/trackpads are SPI-HID devices with Apple-specific
  descriptors and power sequences.
- A Windows HID transport driver or a dedicated input driver is likely needed.
- Until then, use USB input.

### PMU / SMC / battery / RTC

- b00t provides RTC/reset/RNG over HVC for UEFI.
- A Windows driver is needed for battery, charger, thermal sensors, and lid.
- This is also a safety dependency: thermal limits must not rely on macOS.

### I2C / GPIO

- Needed for sensors, Type-C, audio codecs, and some power sequencing.
- Windows has inbox I2C/GPIO frameworks, but Apple controllers need ACPI
  descriptions and possibly a custom driver.

### PCIe

- Apple PCIe host bridges and port controllers need correct ACPI `_CRS`,
  `_DSD`, power resources, and DART descriptions.
- NVMe and USB may be behind PCIe; the first storage path may avoid full PCIe
  enumeration by using a b00t-provided block service.

### Wi-Fi / Bluetooth

- Apple uses Broadcom/Apple-specific chips with firmware and NVRAM blobs.
- Linux uses `brcmfmac`; Windows has no equivalent inbox driver for these
  variants.
- A custom driver is a large reverse-engineering effort. Use USB Ethernet or a
  supported USB Wi-Fi adapter as an interim.

## P2

- DCP display and WDDM display-only driver.
- GPU acceleration (very large; not required for boot).
- Audio (Apple codecs, speaker amps, jack detection).
- Webcam (complex; may remain unsupported).
- Touch ID/SEP (likely out of scope for an open project).
- Sleep/standby and advanced power management.
- Fan control and thermal policy tuning.
- Firmware update/capsule support.

## Driver signing strategy

Development:

- Windows test signing (`bcdedit /set testsigning on`) and a test certificate.
- Document that this weakens security and is not a user-facing solution.

Production:

- Prefer inbox drivers and ACPI-described hardware.
- For custom drivers, pursue Microsoft attestation/WHQL signing or a supported
  enterprise signing path.
- HAL extensions are SoC-specific and loaded through ACPI CSRT resource groups;
  the exact API is not publicly documented. Treat this as high-risk research,
  not a Phase 1 dependency.

## DART ownership models

Model A: b00t owns all DART domains.

- Windows sees a simple block device or a paravirtual service.
- Safest for bring-up, but not a normal Windows storage stack.

Model B: a Windows miniport owns one DART domain.

- b00t programs the domain's base/stream table and grants the miniport a
  register window.
- The miniport must use 16 KiB-aligned buffers/bounce buffers.
- Best long-term path for native NVMe performance.

Model C: expose IORT/SMMU.

- Not recommended until the DART model is proven. A wrong IORT can cause
  Windows to misprogram the IOMMU and corrupt memory.

## Test matrix

| Area | Minimum test |
| --- | --- |
| Boot | UEFI shell, Windows PE, first Windows boot, safe mode |
| Storage | Sequential/random I/O, queue depth, reset, pagefile, hibernation |
| Input | USB keyboard, SPI keyboard, trackpad, lid switch |
| Display | GOP modes, basic display, resolution changes, sleep/wake |
| Interrupts | Timer stress, IPI stress, storage IRQ storm, no missed EOI |
| Power | Idle, load, thermal ramp, battery/AC, shutdown/reset |
| Recovery | Remove boot object, restore macOS, reinstall Windows |
