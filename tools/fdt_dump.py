#!/usr/bin/env python3
import argparse
import struct
import sys
from pathlib import Path


FDT_MAGIC = 0xd00d_feed
FDT_BEGIN_NODE = 1
FDT_END_NODE = 2
FDT_PROP = 3
FDT_NOP = 4
FDT_END = 9


def align4(value):
    return (value + 3) & ~3


def read_fdt(path: Path):
    data = path.read_bytes()
    if len(data) < 40:
        raise SystemExit("file is too small to be an FDT")
    magic = struct.unpack_from(">I", data, 0)[0]
    if magic != FDT_MAGIC:
        raise SystemExit(f"bad FDT magic: 0x{magic:08x}")
    total_size = struct.unpack_from(">I", data, 4)[0]
    off_struct = struct.unpack_from(">I", data, 8)[0]
    off_strings = struct.unpack_from(">I", data, 12)[0]
    size_strings = struct.unpack_from(">I", data, 32)[0]
    size_struct = struct.unpack_from(">I", data, 36)[0]
    return data, total_size, off_struct, size_struct, off_strings, size_strings


def cstr(data, offset, limit):
    end = data.find(b"\x00", offset, limit)
    if end < 0:
        return None
    return data[offset:end].decode("utf-8", "replace")


def dump(path: Path):
    data, total_size, off_struct, size_struct, off_strings, size_strings = read_fdt(path)
    struct_end = off_struct + size_struct
    strings_end = off_strings + size_strings
    off = off_struct
    depth = 0
    address_cells = 2
    size_cells = 2

    print(f"FDT: size={total_size} struct={size_struct} strings={size_strings}")

    while off + 4 <= struct_end:
        token = struct.unpack_from(">I", data, off)[0]
        off += 4
        if token == FDT_BEGIN_NODE:
            name = cstr(data, off, struct_end) or ""
            off = align4(off + len(name) + 1)
            prefix = "  " * depth
            print(f"{prefix}{name or '/'}")
            depth += 1
        elif token == FDT_END_NODE:
            depth = max(depth - 1, 0)
        elif token == FDT_PROP:
            if off + 8 > struct_end:
                break
            length = struct.unpack_from(">I", data, off)[0]
            name_off = struct.unpack_from(">I", data, off + 4)[0]
            off += 8
            value = data[off:off + length]
            prop_name = cstr(data, off_strings + name_off, strings_end) or "?"
            off = align4(off + length)

            if depth > 0 and prop_name in ("#address-cells", "#size-cells") and len(value) >= 4:
                cells = struct.unpack_from(">I", value, 0)[0]
                if prop_name == "#address-cells":
                    address_cells = cells
                else:
                    size_cells = cells

            if prop_name == "reg" and len(value) >= (address_cells + size_cells) * 4:
                regs = []
                pos = 0
                cell_bytes = (address_cells + size_cells) * 4
                while pos + cell_bytes <= len(value):
                    addr = int.from_bytes(value[pos:pos + address_cells * 4], "big")
                    size = int.from_bytes(value[pos + address_cells * 4:pos + cell_bytes], "big")
                    regs.append((addr, size))
                    pos += cell_bytes
                print(f"{'  ' * depth}{prop_name}: " + ", ".join(f"0x{a:x}+0x{s:x}" for a, s in regs))
            elif prop_name in ("compatible", "model", "serial-number", "stdout-path", "linux,stdout-path") and value:
                text = value.split(b"\x00")[0].decode("utf-8", "replace")
                if text:
                    print(f"{'  ' * depth}{prop_name}: {text}")
            elif len(value) <= 16:
                print(f"{'  ' * depth}{prop_name}: {value.hex()}")
        elif token == FDT_NOP:
            pass
        elif token == FDT_END:
            break
        else:
            print(f"unknown token 0x{token:08x} at offset 0x{off - 4:x}")
            break


def main():
    parser = argparse.ArgumentParser(prog="fdt_dump")
    parser.add_argument("fdt", help="path to an FDT/DTB file")
    args = parser.parse_args()
    dump(Path(args.fdt))


if __name__ == "__main__":
    raise SystemExit(main())
