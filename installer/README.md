# winm-install

`winm-install` is the planned installer for WinM. It is a Python 3 CLI with no
third-party Python dependencies. The current implementation is a scaffold: it
checks the host, prints a plan, downloads/verifies media, and documents
recovery. It does not modify disks.

## Commands

```sh
python3 winm_install.py --workdir ~/winm-work check
python3 winm_install.py --workdir ~/winm-work plan
python3 winm_install.py --workdir ~/winm-work download \
  --url https://example.invalid/windows-arm64.iso \
  --sha256 <hash> \
  --output ~/winm-work/windows-arm64.iso
python3 winm_install.py --workdir ~/winm-work prepare --iso ~/winm-work/windows-arm64.iso
python3 winm_install.py --workdir ~/winm-work install
python3 winm_install.py --workdir ~/winm-work uninstall
python3 winm_install.py --workdir ~/winm-work recovery
```

The global `--workdir` option must come before the subcommand.

## Planned installation flow

1. Verify the Mac model, macOS version, free space, and recovery options.
2. Download an official Windows 11 ARM64 ISO or build one locally from UUP.
3. Verify SHA-256 and refuse untrusted or incomplete media.
4. Resize APFS using the asahi-installer flow, with a full backup requirement.
5. Create an EFI System Partition and an NTFS Windows volume.
6. Apply `install.wim` with `wimlib-imagex` and `ntfs-3g`.
7. Patch `boot.wim`/WinPE for the setup bypass and unattend configuration.
8. Inject WinM test drivers and registry settings.
9. Install m1n1 + b00t as a custom boot object with `kmutil`.
10. Create BCD/Windows Boot Manager entries with `bcdboot` in a disposable
    Windows VM or WinPE.
11. Capture a recovery snapshot and test an uninstall on a sacrificial machine.

## Safety rules

- Never delete or resize a volume without an explicit plan and confirmation.
- Never redistribute Windows binaries.
- Refuse to run `install` until `uninstall` and recovery are tested.
- Keep the macOS recovery partition intact.
- Require a second Mac or Apple Configurator for DFU recovery.

## Missing pieces

- APFS resize integration and rollback.
- ISO/UUP media validation.
- Offline WIM/registry/driver injection.
- BCD creation.
- Boot-object installation and removal.
- Disk layout and partition alignment.
- Secure Boot and signing strategy.
