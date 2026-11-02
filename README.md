# WinM

WinM is an open-source research project to boot Windows 11 ARM64 on Apple
Silicon bare metal. The first target is the 13-inch M3 MacBook Air (`t8122`);
the design is intended to extend to other M-series SoCs.

WinM is not affiliated with Apple, Microsoft, Asahi Linux, or the m1n1 project.
It does not distribute Windows images, Apple firmware, or proprietary drivers.

## Status

Phase 0 research is complete. Phase 1 is a design-and-scaffold phase: establish
the boot ABI, build a Rust `b00t` stage-2 payload, and bring up a minimal UEFI
environment with a serial/framebuffer console.

Nothing in this repository currently produces a bootable Windows installation.
Do not use it on a machine that contains data you cannot lose.

## Target boot chain

```text
Apple iBoot
  -> m1n1 stage 1 (custom boot object on the macOS boot volume)
  -> m1n1 stage 2 (Apple Device Tree -> FDT)
  -> b00t (WinM stage 2, EL2 resident)
  -> EDK2-based UEFI platform (EL1)
  -> Windows Boot Manager (bootmgfw.efi)
  -> Windows loader / ntoskrnl
```

`b00t` receives the m1n1-generated FDT in `x0`, initializes its own MMU and
exception vectors, and remains resident at EL2. It owns Apple-specific platform
services (AIC, timers, PSCI, DART, RTC, RNG, reset, boot policy) and exposes
them to the EL1 UEFI/Windows environment through a documented HVC ABI.

## Repository layout

| Path | Purpose |
| --- | --- |
| `docs/` | Architecture, roadmap, hard problems, driver plan, debugging, risks, legal notes |
| `docs/decisions/` | Architecture decision records |
| `b00t/` | Rust `no_std` stage-2 payload and EL2 platform services |
| `uefi/` | EDK2 Apple Silicon platform package notes and scaffolding |
| `hal-aic/` | Windows AIC HAL extension research/stub |
| `drivers/` | Windows driver skeletons and notes (NVMe first) |
| `installer/` | `winm-install` CLI skeleton |
| `tools/` | Image, FDT, ACPI, and debugging helpers |

## Build b00t

Requirements: Rust with the `aarch64-unknown-none-softfloat` target, `ld.lld`,
and `llvm-objcopy`.

```sh
rustup target add aarch64-unknown-none-softfloat
cd b00t
cargo build --release
llvm-objcopy -O binary \
  target/aarch64-unknown-none-softfloat/release/b00t \
  target/b00t.img
python3 ../tools/pad_b00t.py target/b00t.img
```

The resulting `b00t.img` is an ARM64 Linux Image-compatible payload. m1n1 loads
it at a 2 MiB-aligned address and enters it with the FDT in `x0`.

## Quick start on macOS

```sh
curl -fsSL https://raw.githubusercontent.com/hemu-create/winm/master/install.sh | sh
```

This downloads the installer scaffold and runs host checks only. It does not
modify disks.

## Safety and recovery

Installing a custom boot object changes how the machine boots and may require
recovery mode, a second Mac, or Apple Configurator. The Phase 1 workflow must be
tested on a sacrificial machine first. The installer is not ready for general
use.

## Licensing

Original WinM code and documentation are MIT licensed. m1n1 is
GPL-2.0-or-later; WinM does not copy m1n1 code into MIT-licensed components and
uses a documented boot ABI instead. Any future GPL-derived component must be
kept in a clearly marked GPL-compatible directory. See `docs/legal.md`.

## Sources

- Asahi Linux M3 support: https://asahilinux.org/2026/09/m2-episode-1
- AICv3 binding discussion: https://lkml.iu.edu/2602.2/09020.html
- Apple platform quirks: https://asahilinux.org/docs/platform/quirks/
- m1n1 hypervisor: https://asahilinux.org/docs/sw/m1n1-hypervisor
- Asahi U-Boot: https://asahilinux.org/docs/sw/u-boot
- Windows ACPI/HAL extensions: https://learn.microsoft.com/en-us/windows-hardware/drivers/bringup/acpi-system-description-tables
