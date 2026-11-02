# Phase 3: Installer Hardening and Durable Installation

Status: implementation starting.

## Goal

Make WinM installation repeatable, reversible, and safe enough for a user who
has a backup and a recovery plan. Phase 3 turns the experimental Phase 2 flow
into an installer that can install and remove Windows without destroying macOS.

## Exit criteria

- A fresh install completes on a sacrificial machine without a second computer.
- Uninstall restores macOS boot and disk layout.
- The installer refuses unsafe configurations.
- Windows Update does not immediately break boot.
- Recovery is documented and tested.

## Disk layout

Target layout after installation:

```text
0: GUID Partition Table
1: Apple APFS container (resized, macOS preserved)
2: EFI System Partition (FAT32, WinM + Windows boot files)
3: Microsoft Reserved Partition (MSR, optional but recommended)
4: Windows NTFS volume
5: Recovery / WinRE partition (optional)
```

Rules:

- Never delete or resize a partition that is not explicitly recognized.
- Never shrink APFS below a user-provided or calculated minimum.
- Keep macOS recovery and any unknown partitions intact.
- Align partitions to 1 MiB boundaries.
- Record the original partition table before changes.
- Verify free space and filesystem health before resizing.

## APFS resize strategy

Reuse asahi-installer's proven flow rather than reimplementing APFS internals.
The installer should:

1. Require a full backup.
2. Run `diskutil`/asahi-installer checks.
3. Print a dry-run plan with exact byte ranges.
4. Require explicit confirmation.
5. Resize the APFS container.
6. Create ESP, MSR, and NTFS volumes.
7. Verify all filesystems.
8. Write a recovery manifest.

If any step fails, stop and print recovery instructions. Do not continue with a
partial layout.

## Recovery manifest

Store a JSON manifest in the work directory and on the ESP:

- machine model and SoC;
- original partition table;
- created partition UUIDs;
- m1n1 boot object path and status;
- installer version and build IDs;
- SHA-256 of downloaded media;
- timestamp.

Uninstall uses this manifest to restore the original layout as closely as
possible.

## Boot object management

Installation:

1. Install m1n1 stage 1 as a custom boot object.
2. Place b00t + EDK2 + Windows Boot Manager on the ESP.
3. Create BCD entries.
4. Set boot policy and test signing.
5. Verify a reboot reaches Windows Boot Manager.

Removal:

1. Boot into macOS recoveryOS.
2. Remove the custom boot object.
3. Restore the macOS startup disk.
4. Remove WinM partitions or leave them for data recovery.
5. Reset security policy if required.

The installer's `uninstall` command must print the exact commands and require
confirmation. It must not attempt to remove unknown partitions.

## Offline image preparation

The installer prepares Windows media without redistributing it:

- Download official ISO or build from UUP.
- Verify SHA-256.
- Extract `sources/install.wim` and `sources/boot.wim`.
- Apply LabConfig bypass keys to `boot.wim` WinPE registry:
  - `BypassTPMCheck`
  - `BypassSecureBootCheck`
  - `BypassRAMCheck`
  - `BypassCPUCheck`
  - `BypassStorageCheck`
- Inject WinM test drivers into both WIMs.
- Add unattend.xml.
- Rebuild WIMs with wimlib.
- Apply install.wim to the NTFS volume.

Registry edits require `hivex` or a disposable Windows VM/WinPE. If neither is
available, the installer must stop with a clear message rather than silently
skip a required step.

## Driver injection

Development:

- Test-signed drivers only.
- Document the security impact.
- Keep driver packages in a versioned directory.
- Record driver hashes in the recovery manifest.

Production direction:

- Prefer inbox drivers and ACPI-described hardware.
- Use Microsoft attestation/WHQL signing for custom drivers.
- Keep HAL extensions as a research branch until the interface is validated.

## BCD and boot entries

- Create BCD with `bcdboot` in a Windows VM or WinPE.
- Add Boot#### and BootOrder entries.
- Enable test signing for development.
- Store BCD on the ESP and keep a backup.
- Validate boot entries after every installer run.

## Update policy

Windows Update can replace boot files or change driver state. Phase 3 must:

- Document which updates are known to break WinM.
- Provide a recovery script that restores BCD and boot files.
- Keep a known-good Windows image or snapshot for rollback.
- Test cumulative updates before declaring Phase 3 complete.

## Security posture

- Phase 3 may still use test signing and no Secure Boot.
- Document the risk clearly.
- Do not present test mode as a product feature.
- Begin designing a UEFI Secure Boot key hierarchy and a legitimate driver
  signing path.

## Tooling

- `disk_plan.py`: non-destructive layout planner.
- `recovery_manifest.py`: create and validate manifests.
- `wim_patch.py`: wrap wimlib/hivex operations and refuse partial runs.
- `boot_object.py`: wrap kmutil configure-boot and removal.

## Go/no-go for Phase 4

Proceed only if:

- Install and uninstall are tested on a sacrificial machine.
- Windows Update does not break boot in the tested configuration.
- Recovery from a bad install is documented and repeatable.
- The user can perform recovery without a second Mac in the common case.
