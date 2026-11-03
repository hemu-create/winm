# WinM Apple NVMe KMDF driver

This is the Windows bring-up driver for the Apple NVMe controller. It maps the
controller BAR, checks the NVMe CAP/VS registers, and exposes
`IOCTL_WINM_NVME_GET_INFO` for a probe. It is not a Storport miniport yet and
does not expose a disk.

## Build

Install Visual Studio 2022, the Windows SDK, and the WDK. Then:

```powershell
msbuild drivers\nvme\windows\winm_nvme.vcxproj `
  /p:Configuration=Release `
  /p:Platform=ARM64 `
  /p:WDKContentRoot="C:\Program Files (x86)\Windows Kits\10\"
```

The GitHub Actions `wdk-driver` job builds the same ARM64 driver and uploads the
resulting `.sys`, `.inf`, and catalog.

## Install

The INF binds to `ACPI\WINM0002`. b00t's ACPI must expose the Apple NVMe
controller with that hardware ID and a memory resource for BAR0.

## Next

- Allocate admin queues with 16 KiB-aligned DART-aware common buffers.
- Submit Identify Controller and Identify Namespace.
- Add I/O queue creation and read/write.
- Move the data path to a Storport miniport so Windows sees a disk.
- Add power, reset, and surprise-removal handling.
