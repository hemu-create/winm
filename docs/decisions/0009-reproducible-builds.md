# ADR 0009: Reproducible Builds

## Status

Accepted for planning.

## Context

Firmware and installer artifacts affect user trust and recovery. Reproducible
builds and SBOMs are necessary for a credible security story.

## Decision

- Pin Rust, EDK2, Python, and tool versions.
- Record source revisions and build flags.
- Publish checksums and build metadata.
- Generate an SBOM for each release.
- Prefer deterministic build steps and avoid embedding timestamps.

## Consequences

- Build scripts become more complex.
- Some tools may not support deterministic output.
- Release process requires additional verification.
