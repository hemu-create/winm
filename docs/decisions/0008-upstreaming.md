# ADR 0008: Upstreaming

## Status

Accepted for planning.

## Context

WinM depends on m1n1, U-Boot, and EDK2. Keeping all changes private increases
maintenance burden and prevents community benefit.

## Decision

- Upstream platform fixes and documentation where appropriate.
- Keep WinM-specific policy, branding, and installer behavior in this repository.
- Respect each upstream project's license and review process.
- Do not upstream code that depends on WinM's private security model.

## Consequences

- Some work will be duplicated between WinM and upstream.
- Upstream acceptance may lag WinM's needs.
- Contributors must understand both WinM and upstream conventions.
