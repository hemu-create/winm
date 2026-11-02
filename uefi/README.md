# EDK2 Apple Silicon Platform

Status: design stub. No EDK2 tree is vendored here yet.

## Goal

Build an EDK2-based UEFI firmware for Apple Silicon that b00t can load at EL1.
It must boot to the UEFI shell, publish valid ACPI tables, provide GOP, offer
persistent UEFI variables, and eventually load Windows Boot Manager.

## Starting point

Use `ArmVirtPkg/ArmVirtQemu.dsc` as the closest reference. Replace QEMU-specific
services:

| QEMU service | WinM replacement |
| --- | --- |
| fw_cfg | b00t handoff blob |
| QEMU GIC/timer descriptions | b00t ACPI tables |
| QEMU variable store | file-backed store on ESP |
| QEMU RTC/reset | b00t HVC services |
| QEMU GOP | simple framebuffer GOP |
| QEMU serial | Apple S5L UART |

Do not start from `UefiPayloadPkg` for ARM64; it is oriented toward x86
universal-payload platforms and does not provide the Apple/ARM platform
services we need.

## Required modules and libraries

- `ArmPlatformLib` for Apple memory map and CPU topology.
- `ArmGicLib` backed by the b00t virtual GICv2.
- `SerialPortLib` for the Apple UART.
- `RngLib` backed by b00t HVC or Apple RNG.
- `ResetSystemLib` backed by PSCI HVC.
- `RealTimeClockLib` backed by b00t.
- `AcpiTableDxe` plus a platform ACPI DXE that installs b00t's tables.
- `VariableRuntimeDxe` plus a file-backed FVB/variable store.
- GOP driver for the m1n1 framebuffer.
- BDS, UEFI shell, and boot manager.
- Optional: `DevicePathDxe`, `PartitionDxe`, `FatPkg`, `ScsiBus`/`NvmExpressDxe`
  once storage is available.

## Handoff blob

b00t passes a versioned, naturally aligned structure containing:

- magic and version;
- FDT pointer and size;
- ACPI RSDP pointer;
- UEFI memory map descriptors;
- framebuffer base/width/height/stride/format;
- UART base;
- HVC function table or function IDs;
- boot policy and debug flags.

EDK2 must validate the magic/version before using it. The blob is reserved in
the UEFI memory map.

## UEFI variables

Phase 1 uses a file-backed store on the ESP:

- path: `\EFI\WinM\vars.fv`;
- backup: `\EFI\WinM\vars.bak`;
- generation counter and checksum;
- flush on `SetVariable` and at `ExitBootServices`;
- recovery from a corrupt primary by restoring the backup.

Validate Boot####, BootOrder, BootNext, and SecureBoot variables across reboot
and power loss.

## Secure Boot

Phase 1 does not enable Secure Boot. Later:

- define a WinM key hierarchy;
- enroll Microsoft KEK/db only where policy permits;
- provide a recovery path for key loss;
- sign WinM drivers and any custom UEFI components;
- document the trust model and limitations.

## Build integration

The EDK2 tree should be a submodule or a pinned external checkout, not copied
into this repository. A build script should:

1. Check out the pinned EDK2 revision.
2. Apply the WinM platform package.
3. Build `DEBUG` and `RELEASE` AARCH64 firmware.
4. Emit the firmware image and a map file.
5. Run a host-side ACPI/table sanity check.

## Phase 1 exit test

- UEFI shell prompt over UART and framebuffer.
- `memmap` shows b00t, ACPI, framebuffer, and firmware reservations.
- `acpidump`/`iasl` accepts the tables.
- `setvar`/`dmpstore` persist across reboot.
- Timer and PSCI tests pass.
