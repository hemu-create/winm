# Phase 4: Drivers and Usability

Status: planning.

## Goal

Make Windows usable as a daily driver on the target M3 MacBook Air. Phase 4 is
where WinM stops being a boot experiment and becomes an operating system
installation, but only after Phase 3 proves that installation and recovery are
safe.

## Exit criteria

- Stable desktop use for a full workday.
- Suspend/resume either works or is cleanly disabled.
- Thermal throttling does not damage hardware or corrupt the OS.
- Driver signing path is documented.
- A user can install common applications and updates.

## Driver roadmap

### Input

- USB keyboard/mouse first.
- SPI-HID keyboard and trackpad driver.
- Lid switch and power button.
- Trackpad gestures and force touch are optional.

### Power and sensors

- PMU/SMC driver for battery, charger, thermal sensors, and fans.
- Thermal policy with conservative limits.
- Battery charging and AC detection.
- RTC already provided by b00t/UEFI.

### Connectivity

- PCIe host bridge and port power.
- Wi-Fi and Bluetooth, likely requiring a large firmware and driver effort.
- USB Ethernet and supported USB Wi-Fi adapters as interim solutions.
- Thunderbolt/USB4 may remain partial.

### Display

- Simple framebuffer through Phase 3.
- DCP display driver for native resolution and brightness.
- WDDM display-only driver before full GPU acceleration.
- Multi-display and notch handling.

### Audio

- Apple audio codec and speaker amplifier.
- Headphone jack detection.
- Microphone and basic DSP.

### Camera and biometrics

- Webcam is a large reverse-engineering effort and may remain unsupported.
- Touch ID/SEP is likely out of scope for an open project.

### Storage and performance

- Native NVMe storport miniport.
- 16 KiB-aligned DMA and interrupt completion.
- Power management and queue recovery.

## Thermal and power safety

- Read thermal sensors continuously.
- Enforce conservative limits before stress testing.
- Keep fans at a safe default on fan-equipped machines.
- Add watchdog and thermal shutdown paths.
- Do not run long benchmarks until thermal behavior is characterized.
- Disable unsupported sleep cleanly rather than risking data loss.

## Performance goals

- Boot to desktop in a reasonable time.
- Responsive UI under normal desktop load.
- Storage throughput sufficient for development.
- Interrupt latency low enough for interactive use.
- No unexplained hangs during a workday.

## Signing and distribution

- Use test signing only for development.
- Move custom drivers toward Microsoft attestation/WHQL.
- Document which drivers require signing exceptions.
- Keep HAL extensions as a research branch until validated.

## User experience

- Boot menu for macOS/Windows/recovery.
- Clear installer warnings and recovery instructions.
- Crash collection and log upload opt-in.
- Documentation for supported and unsupported features.

## Test matrix

| Area | Minimum test |
| --- | --- |
| Boot | UEFI shell, Windows PE, full install, safe mode |
| Storage | Sequential/random I/O, queue depth, reset, pagefile, hibernation |
| Input | USB keyboard, SPI keyboard, trackpad, lid switch |
| Display | GOP modes, native resolution, brightness, sleep/wake |
| Interrupts | Timer stress, IPI stress, storage IRQ storm, no missed EOI |
| Power | Idle, load, thermal ramp, battery/AC, shutdown/reset |
| Recovery | Remove boot object, restore macOS, reinstall Windows |
| Updates | Cumulative update install and reboot |

## Known non-goals

- Full GPU acceleration.
- Touch ID/SEP.
- Perfect MacBook feature parity.
- Support for all Apple Silicon SoCs before the target is stable.

## Go/no-go for Phase 5

Proceed to security and broader SoC work only if:

- Phase 4 exit criteria are met on the target M3 Air.
- Driver signing path is credible.
- Thermal behavior is safe.
- The project has maintainers for the supported hardware.
