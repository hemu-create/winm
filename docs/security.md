# Security, Signing, and Updates

## Development mode

- Test signing is enabled with `bcdedit /set testsigning on`.
- Driver signature enforcement may be disabled for development.
- No Secure Boot in Phase 1-3.
- These modes reduce platform security and are not user-facing features.

## Production direction

- Design a UEFI Secure Boot key hierarchy.
- Enroll Microsoft keys only where policy and licensing permit.
- Sign WinM UEFI components and drivers.
- Use Microsoft attestation/WHQL for custom drivers where possible.
- Keep recovery possible after key loss.

## Driver signing

- Prefer inbox drivers and ACPI-described hardware.
- Custom drivers need a signing path before production use.
- HAL extensions are high risk and require Microsoft-facing validation.
- Record driver hashes and versions in the recovery manifest.

## Windows Update

- Updates can replace boot files or change driver state.
- Maintain a known-good image or snapshot.
- Test cumulative updates before declaring a phase complete.
- Provide recovery scripts that restore BCD and boot files.

## Incident response

- Keep serial logs and crash dumps.
- Document how to remove the boot object and restore macOS.
- Never ship a permanent signature-enforcement bypass.
- Treat any cryptographic key compromise as a critical incident.
