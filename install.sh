#!/usr/bin/env bash
set -euo pipefail

WORKDIR="${WINM_WORKDIR:-$HOME/winm-work}"
mkdir -p "$WORKDIR"

curl -fsSL "https://raw.githubusercontent.com/hemu-create/winm/master/installer/winm_install.py" \
  -o "$WORKDIR/winm_install.py"

cat <<'GUIDE'
WinM installer
==============

This is the simple flow:

1. Check the host and available tools.
2. List disks and partitions.
3. Review the APFS/ESP/NTFS partition plan.
4. Review the Windows/b00t install plan.
5. Confirm before anything modifies a disk.

Run the commands below only after a full backup and a tested recovery path.
GUIDE

echo
exec python3 "$WORKDIR/winm_install.py" --workdir "$WORKDIR" interactive
