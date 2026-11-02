#!/usr/bin/env python3
import argparse
import shutil
import subprocess
from pathlib import Path


def require(tool: str) -> None:
    if shutil.which(tool) is None:
        raise SystemExit(f"missing required tool: {tool}")


def main() -> int:
    parser = argparse.ArgumentParser(prog="wim_patch")
    sub = parser.add_subparsers(dest="command", required=True)

    info = sub.add_parser("info", help="show WIM information")
    info.add_argument("--wim", required=True)

    extract = sub.add_parser("extract", help="extract a WIM image")
    extract.add_argument("--wim", required=True)
    extract.add_argument("--image", type=int, default=1)
    extract.add_argument("--output", required=True)

    args = parser.parse_args()

    require("wimlib-imagex")

    if args.command == "info":
        subprocess.run(["wimlib-imagex", "info", args.wim], check=True)
    else:
        output = Path(args.output)
        output.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["wimlib-imagex", "extract", args.wim, str(args.image), str(output)],
            check=True,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
