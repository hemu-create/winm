# Windows ARM64 ISO tools

`repack-windows.ps1` injects WinM drivers and an unattend file into a
user-supplied Windows 11 ARM64 ISO, then repacks it with `oscdimg`.

Run it on Windows with the Windows ADK Deployment Tools installed:

```powershell
.\repack-windows.ps1 `
  -IsoPath C:\media\Win11_ARM64.iso `
  -OutputIso C:\media\WinM-Win11-ARM64.iso `
  -DriverDir C:\winm\drivers\arm64 `
  -UnattendPath C:\winm\autounattend.xml
```

The script mounts `sources/install.wim` and every image in `sources/boot.wim`,
adds the drivers with `Add-WindowsDriver -ForceUnsigned`, optionally copies
`unattend.xml`, and rebuilds a UEFI-bootable ISO.

## Limits

- Use your own licensed Windows media. Do not redistribute the modified ISO.
  Distribute the scripts and drivers instead.
- Windows ARM64 is built with 4 KiB pages. There is no supported script or
  kernel switch that changes Windows to 16 KiB pages. Apple DART's 16 KiB
  granule is handled by WinM drivers using 16 KiB-aligned DMA buffers and
  bounce buffers.
- Driver injection requires real `.inf` drivers built for ARM64. Unsigned test
  drivers require test signing or a valid signing chain.
- The ISO is not a WinM boot image. Windows still has to boot through
  b00t/EDK2 on the target Mac.
