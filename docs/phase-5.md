# Phase 5: Security, Upstreaming, and Broader SoCs

Status: planning.

## Goal

Move WinM from a research project to a maintainable, secure, and broader
platform. Phase 5 is only appropriate after Phase 4 has produced a stable daily
driver on the target M3 Air.

## Exit criteria

- UEFI Secure Boot is designed and, where possible, enabled.
- A credible driver signing path exists.
- Builds are reproducible and an SBOM is published.
- At least one additional SoC family is supported.
- Upstream contributions have been made where appropriate.
- Recovery and firmware update paths are hardened.

## Secure Boot

- Define a WinM key hierarchy.
- Enroll Microsoft KEK/db only where policy and licensing permit.
- Sign WinM UEFI components.
- Keep a recovery path for key loss.
- Document the trust model and its limitations.
- Do not weaken security to make installation easier.

## Driver signing

- Move custom drivers from test signing to attestation/WHQL.
- Keep HAL extensions as a research branch until validated.
- Prefer inbox drivers and ACPI-described hardware.
- Document signing requirements for contributors.

## Reproducible builds

- Pin all toolchain and dependency versions.
- Record source revisions and build flags.
- Produce deterministic artifacts where possible.
- Publish checksums and build metadata.
- Keep build scripts in version control.

## SBOM

- Generate an SBOM for firmware and installer artifacts.
- Include third-party components and licenses.
- Review licenses before release.
- Keep the SBOM with every published build.

## Broader SoC support

Initial support matrix:

| SoC | Status | Notes |
| --- | --- | --- |
| t8122 (M3) | Primary target | Base M3 Air first |
| t8132 (M3 Pro/Max) | Planned | Thermal and PCIe differences |
| t8112 (M2) | Planned | AICv2, older platform |
| t8103 (M1) | Planned | AICv1, original target |
| t6xxx (M1/M2/M3 Pro/Max/Ultra) | Research | Different memory/PCIe topology |
| t814x (M4) | Research | AICv3 or newer |
| t8152 (M5?) | Research | Unknown differences |

Rules:

- Keep SoC-specific code behind a platform abstraction.
- Use ADT/FDT as the source of truth.
- Do not assume register addresses from one SoC apply to another.
- Add a SoC to the support matrix only after hardware testing.

## Upstreaming

- Contribute platform fixes to m1n1 where appropriate.
- Contribute EDK2 platform improvements.
- Contribute documentation and tooling.
- Keep WinM-specific policy and branding out of upstream contributions.
- Respect each project's license and review process.

## Firmware updates

- Design a capsule update path or a documented manual update flow.
- Preserve recovery after a failed update.
- Version b00t, EDK2, and installer components independently.
- Keep rollback possible.

## Maintenance

- Define supported hardware and OS versions.
- Establish a security reporting process.
- Set expectations for update cadence.
- Keep the project's scope narrow enough to maintain.

## Go/no-go

Phase 5 is complete when:

- Secure Boot and signing are credible.
- At least one additional SoC boots Windows PE.
- Reproducible builds and SBOMs are published.
- Upstream contributions are accepted or in review.
- The project has a documented maintenance plan.
