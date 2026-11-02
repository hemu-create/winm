# Debugging Strategy

Bare-metal bring-up fails in ways that are invisible without a debug channel.
WinM treats observability as a Phase 1 feature, not an afterthought.

## Debug channels

### Apple UART

- m1n1 initializes the debug UART and exposes it through the FDT (`serial0`
  alias, `/arm-io/uart0` or `/arm-io/uart6/debug-console`).
- b00t uses the same UART for early output.
- EDK2 `SerialPortLib` uses the same UART.
- Windows kernel debugging can use serial once the UART is described in SPCR
  and DBG2.
- Use a USB-to-serial adapter and a reliable ground; do not rely on a single
  unverified cable.

### m1n1 proxy

m1n1's proxy client can drive the target over UART or USB. It can inspect
memory, call functions, and run a guest with the m1n1 hypervisor.

```sh
python3 proxyclient/tools/run_guest.py path/to/kernelcache.macho -- \
  "debug=0x14e serial=3 apcie=0xfffffffe -enable-kprintf-spam wdt=-1 clpc=0"
```

KDK symbols:

```sh
python3 proxyclient/tools/run_guest.py path/to/kernelcache.macho \
  -s path/to/DWARF -- "debug=0x14e serial=3"
```

For b00t, the development path is to use a proxy `P_CALL` to set the next-stage
entry and FDT arguments directly, avoiding a full m1n1 rebuild for every b00t
change.

### m1n1 hypervisor

The m1n1 hypervisor is useful for:

- Tracing MMIO accesses from a guest.
- Testing GIC/AIC behavior.
- Observing exception entry/exit.
- Catching accesses to unimplemented devices.

It is not a substitute for bare-metal tests because it changes the exception and
memory environment.

### Framebuffer console

b00t and EDK2 should be able to draw a simple text console into the m1n1
framebuffer. This is the fallback when UART is unavailable or when the UART is
being used for Windows kernel debugging.

### USB CDC-ACM / USB debug

Once USB works, a CDC-ACM virtual UART provides a faster and more convenient
channel than the physical UART. It is also a useful recovery path.

### Windows debugging

- Enable test signing in BCD for development.
- Enable kernel debugging over serial, USB 3.0, or kdnet.
- Configure a crash dump and a known-good WinDbg workspace.
- Keep a serial log of every boot attempt.
- Use Windows PE as the first user-mode environment; it is easier to recover
  than a full installation.

### EDK2 debugging

- Build EDK2 with `DEBUG`/`NOOPT` and serial debug output.
- Use `DebugLib`, `DebugPrint`, and `ASSERT` early.
- Keep a UEFI shell build for poking at ACPI tables, memory map, and GOP.
- Use `acpidump`/`iasl` on a known-good machine to compare table structure.

## Bring-up order

1. b00t entry and UART: print the FDT pointer, magic, size, and SoC type.
2. FDT parse: print memory regions, reserved regions, AIC, UART, framebuffer.
3. MMU: identity-map device MMIO, enable caches, run a memory test.
4. Exceptions: trigger a known exception and verify the vector prints state.
5. AIC: read version/offsets; mask all sources; verify no spurious interrupts.
6. Timer: program a timer, receive FIQ at EL2, count ticks.
7. GICv2: emulate distributor/CPU interface; deliver a virtual timer IRQ to an
   EL1 test payload.
8. PSCI: start a secondary CPU; verify it enters EL1 and reports its MPIDR.
9. EDK2: enter the firmware, reach the shell, print the memory map.
10. ACPI: dump tables from the shell and validate with `iasl -d`.
11. Storage: enumerate NVMe; read/write a scratch LBA.
12. Windows PE: boot from USB/NVMe; capture the first failure.

## Failure triage

| Symptom | First checks |
| --- | --- |
| No UART output | FDT pointer, UART base, clock/power, TXBE polling, cable |
| Immediate exception loop | EL2 vectors, stack alignment, MMU attributes, x18 |
| Hangs after MMU on | TLB/cache maintenance, device mappings, page-table walk |
| Timer never fires | AIC mask, timer FIQ config, HCR_EL2.FMO, GIC state |
| Windows PE hangs early | GIC emulation, timer, ACPI MADT/FADT, memory map |
| Storage timeouts | DART domain, PRP alignment, queue interrupts, reset |
| Boot Manager loops | UEFI variables, BootOrder, BCD, ESP path, Secure Boot |
| Random corruption | 4 KiB/16 KiB DMA mismatch, cache maintenance, reserved memory |

## Logging format

Every b00t log line should include:

- Component (`b00t`, `gic`, `aic`, `psci`, `acpi`, `pci`, `nvme`).
- Severity.
- A monotonically increasing timestamp from the ARM generic counter.
- A stable message ID where possible.

Example:

```text
[    0.000000] b00t: WinM stage 2
[    0.000012] b00t: fdt=0x... size=0x...
[    0.000030] fdt: memory 0x...-0x...
[    0.000044] aic: version=3 ack=0x... cap0=0x... maxirq=...
[    0.000061] gic: v2 emulation ready
[    0.000080] psci: version 1.1 hvc
```

## Recovery

- Keep a macOS recovery partition and a second Mac for DFU/restore.
- Keep the m1n1 stage-1 boot object removable from recoveryOS.
- Test `kmutil configure-boot` removal before trusting the installer.
- Keep a UEFI shell and Windows PE on removable media.
- Never overwrite the only copy of the macOS boot volume.
