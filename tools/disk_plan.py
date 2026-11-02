#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(prog="disk_plan")
    parser.add_argument("--windows-size-gib", type=int, default=128)
    parser.add_argument("--esp-size-mib", type=int, default=512)
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    plan = {
        "partitions": [
            {"name": "APFS container", "type": "apfs", "action": "resize, do not delete"},
            {"name": "EFI System Partition", "type": "fat32", "size_mib": args.esp_size_mib},
            {"name": "Microsoft Reserved Partition", "type": "msr", "size_mib": 16},
            {"name": "Windows", "type": "ntfs", "size_gib": args.windows_size_gib},
            {"name": "Recovery", "type": "ntfs", "size_mib": 1024, "optional": True},
        ],
        "rules": [
            "Never delete unknown partitions.",
            "Require a full backup before resizing APFS.",
            "Align partitions to 1 MiB boundaries.",
            "Record the original partition table in a recovery manifest.",
            "Refuse to run if free space is insufficient.",
        ],
    }

    text = json.dumps(plan, indent=2)
    if args.output:
        Path(args.output).write_text(text + "\n")
        print(f"wrote {args.output}")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
