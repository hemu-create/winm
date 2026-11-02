#!/usr/bin/env python3
import struct
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} <b00t.img>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    image = bytearray(path.read_bytes())
    if len(image) < 0x40:
        print("image is too small to contain an ARM64 header", file=sys.stderr)
        return 1
    if image[0x38:0x3C] != b"ARM\x64":
        print("missing ARM64 Image magic at offset 0x38", file=sys.stderr)
        return 1

    image_size = struct.unpack_from("<Q", image, 0x10)[0]
    if image_size < 0x40 or image_size > 64 * 1024 * 1024:
        print(f"implausible image_size: {image_size}", file=sys.stderr)
        return 1
    if len(image) > image_size:
        print(
            f"raw image ({len(image)} bytes) exceeds header image_size ({image_size})",
            file=sys.stderr,
        )
        return 1

    image.extend(b"\x00" * (image_size - len(image)))
    path.write_bytes(image)
    print(f"padded {path} to {image_size} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
