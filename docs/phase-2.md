# Phase 2: Windows PE and Storage Bring-up

Status: implementation starting.

## Goal

Boot Windows PE / Setup from an internal NVMe partition under test signing on
the target M3 MacBook Air, with enough storage, input, and debug support to
reach `wpeinit` and the disk-partitioning screen.

Phase 2 does not produce a daily-driver installation. It proves that the Phase 1
platform can carry Windows far enough to install.

## Exit criteria

- Windows PE boots to `wpeinit` without an unhandled exception.
- Internal NVMe is visible and readable/writable.
- Setup reaches the disk partitioning screen.
- `bootmgfw.efi` starts `winload.efi` from NVMe.
- The first Windows kernel boot reaches either a desktop or a reproducible
  bugcheck with a useful crash dump.
- A known-good macOS recovery path remains intact.
- Serial logs are captured for every boot attempt.

## Scope

### In scope

- b00t HVC service ABI for RTC, RNG, reset, storage, and variable persistence.
- A b00t-owned NVMe block service as the safest first storage path.
- NVMe controller discovery and 16 KiB-aligned DMA buffers.
- USB xHCI/DWC3 and Type-C input/install-media support.
- SPI-HID or USB keyboard input.
- Windows test signing and driver signature enforcement policy.
- BCD creation and repair workflows.
- Crash/debug path: serial, WinDbg, kdnet/USB debug where practical.
- Installer offline media preparation and unattend configuration.
- Windows PE test harness and boot-log capture.

### Out of scope

- Production Secure Boot.
- WHQL/attestation signing.
- DCP/WDDM display acceleration.
- Wi-Fi/Bluetooth, audio, camera, sleep, thermals beyond safe limits.
- M3 Pro/Max and other SoC families.

## Boot and test loop

1. Build b00t, EDK2, and the test NVMe miniport.
2. Install them on a sacrificial M3 Air using the experimental installer flow.
3. Boot to UEFI shell and verify ACPI, memory map, storage enumeration.
4. Boot Windows PE from USB or internal NVMe.
5. Capture serial output and any crash dump.
6. If Windows PE reaches `wpeinit`, run storage and input tests.
7. If Setup reaches disk selection, create a test partition and install.
8. Validate `bootmgfw.efi` -> `winload.efi` -> `ntoskrnl.exe`.
9. Record the first failure and the exact register/IRQL/bugcheck.

## Storage model

Phase 2A uses a b00t-owned NVMe block service:

- b00t owns the NVMe controller and its DART domain.
- Windows uses a test storport miniport that forwards SCSI requests over HVC.
- All DMA buffers and PRP lists are 16 KiB aligned.
- This proves the Windows storage stack, BCD, and installation flow without
  solving native DMA ownership.

Phase 2B moves toward a native miniport:

- b00t grants a preconfigured DART domain or register window.
- The miniport owns queue pairs and uses 16 KiB-aligned buffers/bounce buffers.
- Interrupt completion is added after polling is stable.
- Surprise reset, large I/O, pagefile, and hibernation are tested.

See `drivers/nvme/README.md` and `docs/decisions/0003-page-size-and-dart.md`.

## Interrupt and timer requirements

- The virtual GICv2 must support the INTIDs that Windows and storport expect.
- NVMe completion interrupts must not be lost or duplicated.
- EOI ordering must match AIC semantics under I/O load.
- Timer drift and IPI behavior must be stable for at least 30 minutes under
  storage load.

## Input

- USB keyboard is the first supported input.
- SPI-HID keyboard/trackpad is a Phase 2 goal but may slip to Phase 4.
- If no internal input works, use a USB keyboard and mouse for Setup.

## Driver signing

- Use Windows test signing for all custom drivers.
- Document the security impact and require a sacrificial machine.
- Do not attempt WHQL/attestation until the storage path is stable.
- Prefer inbox drivers for USB, display, and PCIe where possible.

## Debugging

- Serial output from b00t, EDK2, and Windows.
- Kernel debugging over serial, USB, or kdnet when available.
- Crash dumps stored on a known-good partition.
- A boot-log naming convention that includes SoC, build ID, and timestamp.

## Risks specific to Phase 2

- Windows PE may require GIC behavior that the emulator does not yet provide.
- NVMe/DART alignment may force a slower bounce-buffer path.
- Setup may depend on UEFI variables or ACPI details not yet implemented.
- Test signing may hide driver-loading issues that production signing exposes.
- BCD creation may require a Windows VM/WinPE environment.

## Go/no-go for Phase 3

Proceed to installer hardening only if:

- Windows PE boots repeatedly on the target.
- NVMe survives a 30-minute I/O stress test.
- The recovery path is tested and documented.
- The first Windows installation can be removed and macOS restored.
