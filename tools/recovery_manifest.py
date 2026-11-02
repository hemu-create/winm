#!/usr/bin/env python3
import argparse
import json
import hashlib
import platform
from datetime import datetime, timezone
from pathlib import Path


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(prog="recovery_manifest")
    parser.add_argument("--output", required=True)
    parser.add_argument("--iso", default="")
    parser.add_argument("--partition-table", default="")
    parser.add_argument("--boot-object", default="")
    args = parser.parse_args()

    manifest = {
        "created": datetime.now(timezone.utc).isoformat(),
        "host": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
        },
        "iso": {},
        "partition_table": args.partition_table,
        "boot_object": args.boot_object,
        "installer_version": "0.1.0",
    }

    if args.iso:
        iso = Path(args.iso).expanduser()
        if iso.exists():
            manifest["iso"] = {
                "path": str(iso),
                "sha256": sha256_file(iso),
                "size": iso.stat().st_size,
            }

    output = Path(args.output).expanduser()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
