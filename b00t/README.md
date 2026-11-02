# b00t

`b00t` is the WinM stage-2 payload. It is loaded by m1n1 as an ARM64
Image-compatible blob, entered at EL2 with the m1n1-generated FDT in `x0`, and
is intended to remain resident at EL2 while EDK2 and Windows run at EL1.

This is a scaffold, not a complete bootloader. It currently:

- provides an ARM64 Image header and position-independent entry code;
- zeroes `.bss` and installs a stack;
- validates and parses the FDT;
- finds the debug UART and prints a platform summary;
- parses memory, framebuffer, and AIC nodes;
- builds and checks an ACPI table header and handoff structure;
- exposes stubs for virtual GICv2 and PSCI.

It does not yet set up an MMU, install EL2 vectors, initialize AIC, emulate a
GIC, start secondary CPUs, or enter EDK2.

## Build

```sh
cargo build --release
llvm-objcopy -O binary \
  target/aarch64-unknown-none-softfloat/release/b00t \
  target/b00t.img
python3 ../tools/pad_b00t.py target/b00t.img
```

`pad_b00t.py` pads the raw image to the `image_size` in the ARM64 Image header.
The linker includes `.bss` and the 64 KiB EL2 stack in that size, so m1n1
reserves enough memory even though the raw file does not contain zero-filled
`.bss` bytes.

## Image contract

- The header is at offset 0.
- `code0` branches to `_entry` at offset 0x40.
- `text_offset` is 0.
- `image_size` covers `.text`, `.rodata`, `.data`, `.bss`, and the stack.
- `flags` is 0.
- `ARM\x64` is at offset 0x38.
- No dynamic relocations are currently emitted; keep it that way or add a
  `.rela.dyn` self-relocator before using absolute pointers.

## Modules

| Module | Purpose |
| --- | --- |
| `fdt.rs` | Bounds-checked FDT parsing |
| `uart.rs` | Apple S5L UART polling output |
| `acpi.rs` | ACPI header/RSDP checksum helpers |
| `gic.rs` | Virtual GICv2 register model stub |
| `psci.rs` | PSCI HVC dispatcher stub |
| `handoff.rs` | b00t-to-EDK2 handoff structure |

## Next implementation steps

1. EL2 vector table and exception save/restore, including x18 and TPIDR_EL1.
2. EL2 stage-1 MMU and stage-2 translation.
3. AIC initialization from FDT/ADT offsets and physical IRQ/FIQ handling.
4. Virtual GICv2 distributor/CPU-interface traps.
5. PSCI CPU_ON and secondary CPU bring-up.
6. ACPI table generation and validation.
7. EDK2 handoff and UEFI shell.
