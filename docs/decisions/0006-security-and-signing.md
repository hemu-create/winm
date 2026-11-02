# ADR 0006: Security and Signing

## Status

Accepted for Phase 1/2 planning.

## Context

Windows supports Secure Boot, test signing, and driver signature enforcement.
A custom boot object changes the trust chain. The project must not ship an
insecure permanent bypass.

## Decision

- Phase 1/2: no Secure Boot; Windows test signing for development only.
- Phase 3: design a UEFI Secure Boot key hierarchy and a supported driver
  signing path.
- Never ship a permanent "disable driver signature enforcement" hack as the
  product.
- Treat HAL extensions and custom kernel drivers as signing-sensitive.

## Rationale

- Early bring-up needs fast iteration and diagnostic drivers.
- Test signing is appropriate for a sacrificial development machine but weakens
  platform security.
- Production users need a credible trust story and a recovery path.
- Microsoft signing requirements and UEFI key policy are external dependencies.

## Consequences

- Development images are not suitable for untrusted environments.
- Production signing may require WHQL/attestation or enterprise certificates.
- Secure Boot may limit which Windows images and drivers can boot.
- Recovery must work without the original macOS installation.
