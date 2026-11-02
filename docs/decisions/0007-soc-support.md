# ADR 0007: SoC Support Strategy

## Status

Accepted for planning.

## Context

WinM targets multiple Apple Silicon generations, but hardware differs in AIC
version, DART, PCIe, thermal, and power behavior. Supporting all SoCs at once
would stall Phase 1-4.

## Decision

- Target base M3 (`t8122`) first.
- Keep SoC-specific code behind `platform::Soc`.
- Use ADT/FDT as the source of truth for register addresses and topology.
- Add a SoC to the support matrix only after hardware testing.
- Prefer capability detection over hardcoded SoC checks where possible.

## Consequences

- Early code may include t8122-specific constants, but they must be isolated.
- M1/M2 and M3 Pro/Max support is explicitly later work.
- Unsupported SoCs fail early with a clear message rather than partially booting.
