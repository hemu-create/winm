#!/usr/bin/env python3
import argparse


def main() -> int:
    parser = argparse.ArgumentParser(prog="boot_object")
    sub = parser.add_subparsers(dest="command", required=True)

    install = sub.add_parser("install", help="print kmutil install commands")
    install.add_argument("--stage1", required=True)
    install.add_argument("--volume", required=True)

    remove = sub.add_parser("remove", help="print kmutil removal commands")
    remove.add_argument("--volume", required=True)

    args = parser.parse_args()

    if args.command == "install":
        print("Install m1n1 as a custom boot object:")
        print(f"  kmutil configure-boot -c {args.stage1} --raw --entry-point 2048 --lowest-virtual-address 0 -v {args.volume}")
        print("Verify the volume path and keep a recovery Mac available.")
    else:
        print("Remove the custom boot object from recoveryOS:")
        print(f"  kmutil configure-boot -v {args.volume}")
        print("Then restore the macOS startup disk and security policy.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
