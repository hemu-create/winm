# ADR 0004: m1n1 Payload ABI

## Status

Accepted.

## Context

m1n1 can load several payload formats. b00t needs a stable way to be loaded and
entered, and a way to discover memory, devices, and the framebuffer.

## Decision

Package b00t as an ARM64 Linux Image-compatible payload:

- Branch instruction at offset `0x0`.
- ARM64 Image header fields at the standard offsets.
- `ARM\x64` magic at offset `0x38`.
- Valid `image_size`.
- Loaded at a 2 MiB-aligned address.

m1n1 enters b00t with the generated FDT pointer in `x0`. b00t must be
position-independent or self-relocate, set up its own stack and MMU, and never
return to m1n1.

## Rationale

- m1n1 already recognizes and aligns ARM64 Image payloads.
- The FDT is the authoritative description of memory, reserved regions, AIC,
  UART, and framebuffer.
- This avoids depending on m1n1's internal `boot_args` structure.
- It works with both the m1n1 payload chain and a proxy-driven development loop.

## Consequences

- b00t carries an image header and a small position-independent entry stub.
- b00t cannot assume m1n1's MMU, stacks, or exception vectors remain.
- The build must produce a raw binary whose size matches `image_size`.
- Any use of m1n1 code remains GPL-2.0-or-later; the ABI itself is documented
  and independently implemented.
