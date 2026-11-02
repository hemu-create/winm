# Hard Problems

This document tracks the issues most likely to stop the project. Each item has a
current understanding, a mitigation, and an experiment that can resolve it.
Speculative items are marked as such.

## 1. AIC versus the Windows GIC model

Problem: Apple uses the Apple Interrupt Controller (AIC), not a GIC. Windows
ARM64 has an inbox GIC driver and expects GIC distributor/CPU-interface MMIO.
AICv3 on `t8122` has dynamic offsets and different ack/EOI semantics.

Current understanding:

- AIC is a legitimate interrupt controller with timer and PMU FIQ sources.
- AICv3 is similar to AICv2 with additional offsets supplied by the Apple
  Device Tree (`aic-iack-offset`, `cap0-offset`, `maxnumirq-offset`).
- Windows has inbox GIC support; Raspberry Pi 4 Windows prior art uses GICv2.
- A GICv2 emulation can run entirely at EL2 with stage-2 MMIO traps and
  `HCR_EL2.VI` injection. No GIC virtualization hardware is required.

Mitigation:

- Implement a GICv2 distributor and CPU interface in b00t.
- Map GIC INTIDs to AIC hardware IRQs.
- Forward physical IRQ/FIQ to EL2 and inject virtual IRQs.
- Keep an AIC HAL extension as a later performance path, not the first path.

Experiment:

- Boot a bare-metal EL1 test payload that programs the emulated GIC and waits
  for a timer interrupt.
- Then boot Windows PE and observe which GIC registers Windows touches.

Risk: high until Windows PE proves it accepts the emulation.

## 2. DART 16 KiB granule versus NT 4 KiB pages

Problem: Apple DART uses a 16 KiB granule. Windows ARM64 uses 4 KiB MMU pages.
A driver cannot simply hand arbitrary 4 KiB pages to a DART.

Current understanding:

- Windows uses 4 KiB pages; 64 KiB is allocation granularity, not page size.
- The Apple NVMe controller sits behind DART.
- Exposing an incorrect IORT is worse than exposing no IORT.

Mitigation:

- Use 4 KiB pages for CPU/Windows translation.
- Use 16 KiB-aligned buffers or bounce buffers for DART-backed DMA.
- Keep DART domains owned by b00t initially; provide a narrow storage service or
  a Windows miniport that respects the 16 KiB constraint.
- Do not expose IORT until the SMMU model is validated.

Experiment:

- Measure Windows storport's PRP and buffer alignment assumptions with a
  test-signed miniport.
- Stress mixed 4 KiB/16 KiB I/O and verify DART fault behavior.

Risk: high. This is a likely source of data corruption if rushed.

## 3. Timer and FIQ routing

Problem: Apple timer interrupts are FIQ sources through AIC, while Windows
expects the ARM generic timer and GIC PPIs.

Mitigation:

- Handle the physical timer FIQ at EL2, acknowledge/complete at AIC, and inject
  virtual timer PPIs into the emulated GIC.
- Use `CNTHCTL_EL2`, `CNTVOFF_EL2`, and `HCR_EL2.VI` as appropriate.
- Validate with a periodic EL1 timer test before Windows.

Risk: medium. The mechanism is understood; the exact Apple timer behavior under
guest load needs testing.

## 4. SMP and PSCI

Problem: Apple CPUs do not provide EL3 PSCI. Windows expects PSCI for secondary
CPU bring-up.

Mitigation:

- b00t implements PSCI 1.1 over HVC.
- `CPU_ON` uses the Apple release-register/spin-table path used by m1n1.
- Secondary CPUs enter EL1 with the expected context and are tracked by b00t.

Risk: medium. m1n1 already performs SMP bring-up; reuse the mechanism, not the
code, and document the ABI.

## 5. x18 / TEB ABI

Problem: Windows ARM64 uses x18 for the TEB/KPCR. Apple reserves x18 and macOS
has historically cleared it on context switch or exception return. A bare-metal
EL2 component that clobbers x18 breaks Windows.

Current understanding:

- Apple Silicon hardware supports preserving x18 on exception entry.
- macOS software clears x18; macOS 26.5 adds an "x18 custom ABI" mode with a
  `com.apple.security.custom-x18-abi-toggle` entitlement.
- For bare metal, b00t must preserve x18 and TPIDR_EL1 across all EL2
  exceptions and must not use x18 as a scratch register.

Mitigation:

- Ban x18 from b00t assembly and compiler-generated code.
- Save/restore x18 in EL2 vectors.
- Add a boot-time self-test that writes a pattern to x18, triggers an EL2
  exception, and verifies the pattern survives.

Risk: medium. It is a correctness trap, not necessarily a showstopper.

## 6. UEFI runtime services and variables

Problem: Windows Boot Manager and Windows Setup rely on UEFI variables, RTC,
reset, and a memory map. Apple has no native UEFI variable store.

Mitigation:

- EDK2 file-backed variable store on the ESP for Phase 1.
- b00t-owned RTC/RNG/reset over HVC.
- Runtime memory marked correctly and not reused by Windows.
- Validate Boot####/BootOrder behavior with repeated installs.

Risk: medium. File-backed variables can be corrupted by unclean shutdowns; add a
journal or backup copy.

## 7. EDK2 platform port

Problem: There is no existing Apple Silicon EDK2 platform. `ArmVirtPkg` assumes
QEMU fw_cfg and a virt machine.

Mitigation:

- Build a custom platform package derived from `ArmVirtPkg`/`ArmVirtQemu`.
- Replace fw_cfg with b00t's handoff blob and HVC services.
- Reuse EDK2's ACPI table DXE, GOP, variable, and runtime infrastructure.
- Keep U-Boot only as a Phase 1 scaffolding/loader experiment.

Risk: medium-high. This is a substantial firmware port, not a small patch.

## 8. Windows driver signing

Problem: Test signing is acceptable for development but not for users. HAL
extensions and kernel drivers need Microsoft acceptance or a legitimate
enterprise/developer signing path.

Mitigation:

- Phase 2 uses test signing and documents the security tradeoff.
- Phase 3+ designs a path to WHQL/Microsoft-signed drivers or a supported
  enterprise signing model.
- Do not ship a permanent "disable signature enforcement" hack as the product.

Risk: high for production, low for research.

## 9. Secure Boot and trust chain

Problem: A custom boot object bypasses Apple's normal boot policy. Windows
expects Secure Boot for some features and for a credible security story.

Mitigation:

- Phase 1: no Secure Boot; document the risk.
- Later: UEFI Secure Boot with a WinM-controlled key hierarchy, Microsoft KEK
  enrollment where legally/technically possible, and measured boot.
- Recovery must be possible without the original macOS install.

Risk: high. This is a policy and key-management problem as much as a code
problem.

## 10. Installer and APFS safety

Problem: Resizing APFS and adding partitions can destroy user data. Apple's
volume layout and boot policy are not forgiving.

Mitigation:

- Reuse asahi-installer's proven APFS resize flow with attribution.
- Require a full backup and a dry-run plan.
- Never delete unknown volumes.
- Implement uninstall before calling the installer "ready".
- Test on sacrificial machines and in VMs where possible.

Risk: high. A single bad resize is unacceptable.

## 11. M3-specific reverse engineering

Problem: M3 is newer than M1/M2; some blocks (DCP, DART, PCIe, PMU) may differ.

Current understanding:

- M3 is `t8122`; AICv3 is supported by m1n1 and Linux.
- Base M3 Air is fanless, so thermal behavior differs from Pro/Max laptops.

Mitigation:

- Target the base M3 Air first.
- Use m1n1's ADT/FDT output as the source of truth.
- Keep SoC-specific code behind a platform trait.
- Do not assume M1 register addresses apply to M3.

Risk: medium. AIC/M3 support is no longer the blocker it once was.

## 12. Display and GPU

Problem: Apple DCP is complex and undocumented; Windows needs at least a
framebuffer for Setup.

Mitigation:

- Phase 1/2 use the m1n1 simple framebuffer and Microsoft basic display.
- DCP/WDDM acceleration is Phase 4.
- Do not block installation on GPU acceleration.

Risk: medium for setup, high for daily-driver graphics.

## 13. Networking, audio, camera, sleep

Problem: These are large driver efforts with firmware, power, and regulatory
dependencies.

Mitigation:

- Treat them as post-install work.
- Use USB Ethernet/Wi-Fi/audio as interim solutions.
- Disable unsupported sleep cleanly rather than risking data loss.

Risk: medium. They affect usability, not first boot.

## 14. Thermal and power safety

Problem: macOS's SMC/PMU policy is absent. A bare-metal OS could overheat or
draw unsafe power.

Mitigation:

- Read thermal sensors and enforce conservative limits in b00t.
- Keep fans at a safe default on fanless and fan-equipped machines.
- Add a watchdog and thermal shutdown path before stress testing.
- Do not run long benchmarks until thermal behavior is characterized.

Risk: high if ignored; moderate with conservative limits.
