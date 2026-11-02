#!/usr/bin/env python3
import argparse
import struct
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(prog="b00t_image")
    parser.add_argument("image", help="path to b00t.img")
    args = parser.parse_args()

    data = Path(args.image).read_bytes()
    if len(data) < 0x40:
        raise SystemExit("image is too small")

    code0 = struct.unpack_from("<I", data, 0x00)[0]
    text_offset = struct.unpack_from("<Q", data, 0x08)[0]
    image_size = struct.unpack_from("<Q", data, 0x10)[0]
    flags = struct.unpack_from("<Q", data, 0x18)[0]
    magic = data[0x38:0x3C]

    print(f"code0: 0x{code0:08x}")
    print(f"text_offset: 0x{text_offset:x}")
    print(f"image_size: 0x{image_size:x} ({image_size} bytes)")
    print(f"flags: 0x{flags:x}")
    print(f"magic: {magic!r}")
    print(f"file_size: {len(data)} bytes")
    if magic != b"ARM\x64":
        raise SystemExit("missing ARM64 Image magic")
    if len(data) != image_size:
        print("warning: file size does not match image_size")
    if code0 >> 26 != 0b000101 and code0 >> 26 != 0b000100:
        print("warning: code0 does not look like a branch instruction")


if __name__ == "__main__":
    raise SystemExit(main())
