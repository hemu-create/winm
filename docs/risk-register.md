# Risk Register

Likelihood and impact are qualitative: low, medium, high, critical. This table
is reviewed at the end of each phase.

| ID | Risk | Likelihood | Impact | Mitigation | Status |
| --- | --- | --- | --- | --- | --- |
| R1 | Windows rejects emulated GICv2 | Medium | Critical | Boot Windows PE early; compare GIC register traces; keep HAL extension research parallel | Open |
| R2 | DART 16K/4K mismatch corrupts memory | High | Critical | Bounce buffers, 16K-aligned DMA, b00t-owned domains, no IORT until proven | Open |
| R3 | EL2 exception/timer design unstable | Medium | High | Small vectors, timer stress, m1n1 hypervisor tracing, watchdog | Open |
| R4 | x18/TPIDR_EL1 corruption breaks Windows | Medium | High | Ban x18 in b00t, save/restore in vectors, boot self-test | Open |
| R5 | EDK2 Apple platform port is too large | High | High | Reuse ArmVirtPkg, replace only platform services, stage deliverables | Open |
| R6 | UEFI variable store corrupts on power loss | Medium | High | File-backed store with backup/journal, fsync, recovery tool | Open |
| R7 | APFS resize destroys user data | Low if careful | Critical | Reuse asahi-installer, backup requirement, dry run, uninstall first | Open |
| R8 | Driver signing cannot be made legitimate | High | High | Test signing for dev; plan WHQL/attestation; prefer inbox drivers | Open |
| R9 | Secure Boot/trust chain impossible or weak | High | High | Document test mode; design key hierarchy later; do not ship insecure bypass | Open |
| R10 | Thermal/power damage on fanless M3 Air | Medium | Critical | Conservative thermal limits, watchdog, short tests, sensor validation | Open |
| R11 | M3 block differences break drivers | Medium | Medium | ADT/FDT as source of truth; platform abstraction; test on target | Open |
| R12 | No usable internal storage path | Medium | Critical | b00t block service fallback; USB install media; DART ownership model A | Open |
| R13 | USB/input unavailable during Setup | Medium | High | USB xHCI/Type-C early; USB keyboard; SPI-HID later | Open |
| R14 | Display unusable before DCP driver | Low | High | Simple framebuffer/GOP + Microsoft basic display | Mitigated for setup |
| R15 | Windows Update breaks custom boot | High | Medium | Document update policy; recovery; test cumulative updates | Open |
| R16 | Installer cannot recover macOS boot policy | Medium | Critical | Test kmutil removal; recoveryOS instructions; second Mac | Open |
| R17 | Legal challenge over Windows/Apple redistribution | Low if policy followed | Critical | Never redistribute; user-supplied ISO; clean-room code | Controlled |
| R18 | m1n1 GPL contamination | Medium | Medium | Keep b00t independent; document ABI; isolate GPL code | Controlled |
| R19 | No hardware access for testing | Medium | High | Secure sacrificial M3 Air; remote serial; reproducible images | Open |
| R20 | Project scope expands beyond available effort | High | Medium | Phase gates; stop/go criteria; narrow first target | Open |

## Assumptions to validate

- The target M3 Air can run m1n1 and a custom boot object with the user's
  security policy.
- Windows 11 ARM64 can boot from an internal NVMe namespace exposed by a
  storport miniport or b00t block service.
- A 4 KiB stage-2 mapping is sufficient for Windows and EDK2.
- The Apple timer can be virtualized through AIC and `HCR_EL2.VI` without
  unacceptable latency.
- A file-backed UEFI variable store is acceptable to Windows Boot Manager.
- The user can recover macOS without wiping the Windows installation.

## Review triggers

Review this register when:

- Windows PE first boots.
- The first storage write succeeds.
- The installer performs its first real APFS resize.
- A driver is submitted for signing.
- A new SoC family is added.
- A security researcher reports an issue.
