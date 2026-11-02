#!/usr/bin/env python3
import argparse
import json
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path


def tool_version(name: str) -> str:
    path = shutil_which(name)
    if path is None:
        return "missing"
    try:
        output = subprocess.check_output([name, "--version"], text=True, stderr=subprocess.STDOUT)
        return output.strip().splitlines()[0]
    except Exception as exc:
        return f"error: {exc}"


def shutil_which(name: str):
    import shutil

    return shutil.which(name)


def main() -> int:
    parser = argparse.ArgumentParser(prog="sbom")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    import shutil

    components = [
        {"name": "winm-b00t", "type": "firmware", "license": "MIT"},
        {"name": "winm-installer", "type": "application", "license": "MIT"},
        {"name": "m1n1", "type": "bootloader", "license": "GPL-2.0-or-later", "role": "external"},
        {"name": "EDK2", "type": "firmware", "license": "BSD-2-Clause-Patent", "role": "external"},
        {"name": "rustc", "type": "toolchain", "version": tool_version("rustc")},
        {"name": "cargo", "type": "toolchain", "version": tool_version("cargo")},
        {"name": "ld.lld", "type": "toolchain", "version": tool_version("ld.lld")},
        {"name": "python", "type": "runtime", "version": platform.python_version()},
    ]

    sbom = {
        "created": datetime.now(timezone.utc).isoformat(),
        "format": "winm-sbom-0.1",
        "host": platform.platform(),
        "components": components,
    }

    output = Path(args.output).expanduser()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(sbom, indent=2) + "\n")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
