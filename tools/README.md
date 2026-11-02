# Tools

Host-side helpers for building and debugging WinM images.

## pad_b00t.py

Pads a raw b00t image to the `image_size` stored in its ARM64 Image header.
The linker includes `.bss` and the EL2 stack in `image_size`, so m1n1 reserves
enough memory even though `llvm-objcopy -O binary` does not emit NOBITS bytes.

```sh
python3 tools/pad_b00t.py b00t/target/b00t.img
```

## Planned tools

- `fdt_dump.py`: parse an m1n1 FDT dump and print memory, reserved-memory, AIC,
  UART, and framebuffer nodes.
- `acpi_check.py`: validate RSDP/FADT/MADT/GTDT checksums and basic table
  lengths.
- `b00t_image.py`: inspect the ARM64 Image header, image size, and entry branch.
- `serial_log.py`: timestamp and filter b00t/EDK2/Windows serial logs.
- `disk_plan.py`: print a non-destructive APFS/ESP/NTFS layout plan.
- `acpi_check.py`: validate RSDP and ACPI table checksums/lengths.
- `fdt_dump.py`: print FDT nodes, compatible strings, and register ranges.
- `b00t_image.py`: inspect an ARM64 b00t Image header.
