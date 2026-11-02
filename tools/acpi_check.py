#!/usr/bin/env python3
import argparse
import struct
import sys
from pathlib import Path


def checksum(data: bytes) -> int:
    return sum(data) & 0xff


def read_table(path: Path, offset: int, length: int) -> bytes:
    with path.open("rb") as handle:
        handle.seek(offset)
        return handle.read(length)


def check_header(name: str, data: bytes) -> int:
    if len(data) < 36:
        print(f"{name}: too short ({len(data)} bytes)")
        return 1
    signature = data[0:4]
    length = struct.unpack_from("<I", data, 4)[0]
    if length > len(data):
        print(f"{name}: header length {length} exceeds available {len(data)}")
        return 1
    table = data[:length]
    print(f"{name}: signature={signature!r} length={length} checksum={checksum(table)}")
    if checksum(table) != 0:
        print(f"  bad checksum")
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(prog="acpi_check")
    parser.add_argument("--rsdp", default="")
    parser.add_argument("--xsdt", default="")
    parser.add_argument("--fadt", default="")
    parser.add_argument("--madt", default="")
    parser.add_argument("--gtdt", default="")
    parser.add_argument("--spcr", default="")
    parser.add_argument("--dbg2", default="")
    args = parser.parse_args()

    status = 0
    for name, path in [
        ("XSDT", args.xsdt),
        ("FADT", args.fadt),
        ("MADT", args.madt),
        ("GTDT", args.gtdt),
        ("SPCR", args.spcr),
        ("DBG2", args.dbg2),
    ]:
        if path:
            status |= check_header(name, Path(path).read_bytes())

    if args.rsdp:
        data = Path(args.rsdp).read_bytes()
        if len(data) < 36:
            print("RSDP: too short")
            status = 1
        else:
            print(f"RSDP: checksum20={checksum(data[:20])} checksum36={checksum(data[:36])}")
            if checksum(data[:20]) != 0 or checksum(data[:36]) != 0:
                print("  bad RSDP checksum")
                status = 1

    return status


if __name__ == "__main__":
    raise SystemExit(main())
