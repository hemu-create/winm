# Roadmap

Timelines are estimates for a small, focused team with access to sacrificial
hardware. They are not commitments.

## Phase 0: Research and feasibility (complete)

Goal: establish the platform facts before writing a bootloader.

Deliverables:

- Asahi/m1n1 capabilities, licensing, and boot ABI understood.
- M3/AICv3 support confirmed.
- Apple vs Windows interrupt, timer, page-size, and DMA constraints documented.
- Windows HAL extension, GICv2 prior art, and installer constraints researched.
- Initial architecture, risk register, and decision records.
- No code that pretends to be production-ready.

Exit criteria:

- We can explain how control reaches an EL1 UEFI environment.
- We can name the unknown interfaces and the experiment that resolves each one.
- The project has a defensible first target and a stop/go decision.

Status: complete. See `docs/phase-0.md`.

## Phase 1: b00t and minimal UEFI (design completed, implementation next)

Goal: boot a minimal EDK2 UEFI environment from b00t on the target M3 Air and
show a serial/framebuffer console.

Scope:

- ARM64 Image payload format and self-relocation.
- FDT parser and platform discovery.
- EL2 vectors, MMU, stage-2 translation, and UART.
- AIC initialization and physical interrupt handling.
- Virtual GICv2 distributor/CPU interface.
- PSCI over HVC and secondary CPU bring-up.
- ACPI table handoff and EDK2 platform package.
- Framebuffer GOP.
- File-backed UEFI variables.
- Recovery/test harness.

Out of scope:

- Windows installation.
- Real storage or USB drivers.
- Secure Boot production keys.
- GPU acceleration.
- Sleep, thermals, and battery management.

Milestones:

1. `b00t.img` builds and is accepted by m1n1 as an ARM64 Image payload.
2. b00t prints its FDT summary over UART and parks cleanly.
3. b00t sets up EL2 MMU and stage-2 translation, then enters a trivial EL1
   payload.
4. AIC and virtual GICv2 deliver a timer interrupt to an EL1 test payload.
5. EDK2 `ArmVirtPkg`-derived platform boots to the UEFI shell with serial
   output.
6. EDK2 publishes ACPI tables and a GOP; Linux or a UEFI ACPI checker accepts
   them.
7. UEFI variables persist across reboot.
8. PSCI brings up all performance cores.

Exit criteria:

- UEFI shell prompt over UART and visible in the framebuffer.
- ACPI tables pass a table/AML validator.
- Two or more CPUs online.
- Timer interrupts work for at least 10 minutes without a hang.
- No writes outside reserved regions.
- Recovery path documented and tested.

## Phase 2: Windows PE and storage bring-up

Goal: boot Windows PE / Setup from an internal NVMe partition under test
signing.

Scope:

- NVMe storport miniport and DART ownership.
- USB xHCI/DWC3 and Type-C PD for install media and input.
- UEFI GOP sufficient for Windows Setup.
- UEFI variable and boot-entry behavior.
- BCD creation via `bcdboot` in a disposable Windows VM/WinPE.
- Test-signing BCD option and driver signature enforcement disabled for
  development.
- Basic keyboard input through SPI-HID or USB.
- Crash/debug path: serial, WinDbg, kdnet or USB debug.

Milestones:

1. Windows PE boots to `wpeinit`.
2. Internal NVMe is visible and readable/writable.
3. Setup reaches the disk partitioning screen.
4. `bootmgfw.efi` starts `winload.efi` from NVMe.
5. First Windows kernel boot reaches a bugcheck or desktop with basic display.
6. Automated serial-log capture for every boot attempt.

Exit criteria:

- Windows PE and a first-stage Windows install boot reliably on the target.
- Storage I/O survives a 30-minute stress test.
- A known-good macOS recovery path remains intact.
- Test-signed drivers load without disabling signature enforcement globally
  beyond the documented test mode.

## Phase 3: Installer and durable installation

Goal: a repeatable, reversible installation of Windows 11 ARM64 onto an Apple
Silicon Mac.

Scope:

- APFS resize orchestration and safety checks.
- Windows ISO download/verification or UUP build.
- WIM apply, offline registry edits, unattend, driver injection.
- ESP/NTFS layout, BCD, boot menu.
- Uninstall and recovery.
- User-facing warnings and dry-run plans.
- Secure Boot roadmap and signing strategy.

Exit criteria:

- Fresh install completes on a sacrificial machine without a second computer.
- Uninstall restores macOS boot and disk layout.
- Installer refuses unsafe configurations and never deletes unknown volumes.
- Windows Update does not immediately break boot.

## Phase 4: Drivers and usability

Goal: make Windows usable as a daily driver on the target hardware.

Scope:

- SPI-HID keyboard/trackpad, PMU/SMC, battery, RTC, I2C/GPIO.
- PCIe, Wi-Fi/BT (likely complex firmware/regulatory work).
- DCP display and a WDDM display-only driver.
- Audio, webcam, thermals, power management, sleep.
- Performance tuning and interrupt latency.

Exit criteria:

- Stable desktop use for a full workday.
- Suspend/resume either works or is cleanly disabled.
- Thermal throttling does not damage hardware or corrupt the OS.
- Driver signing path is documented.

## Phase 5: Security, upstreaming, and broader SoCs

Goal: production-quality trust and maintainability.

Scope:

- UEFI Secure Boot with a documented key hierarchy.
- Microsoft-signed HAL extension or a production AIC driver path.
- Reproducible builds and SBOM.
- Upstream contributions to EDK2, m1n1, and Windows driver samples where
  possible.
- M1/M2/M3 Pro/Max/M4/M5 support.
- Firmware update and recovery hardening.

## Stop/go gates

Stop and reassess if any of these fail:

- Gate A: Windows PE cannot tolerate an emulated GICv2 after a reasonable
  debugging effort.
- Gate B: DART ownership cannot be made safe for NT 4 KiB DMA.
- Gate C: The installer cannot guarantee a reversible APFS change.
- Gate D: Thermal/power behavior risks hardware damage.
- Gate E: Driver signing cannot be made legitimate for users.
- Gate F: x18 or another ABI mismatch cannot be preserved on M3.
