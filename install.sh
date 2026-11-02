#!/usr/bin/env bash
set -euo pipefail

WORKDIR="${WINM_WORKDIR:-$HOME/winm-work}"
mkdir -p "$WORKDIR"

curl -fsSL "https://raw.githubusercontent.com/hemu-create/winm/master/installer/winm_install.py" \
  -o "$WORKDIR/winm_install.py"

python3 "$WORKDIR/winm_install.py" --workdir "$WORKDIR" check
python3 "$WORKDIR/winm_install.py" --workdir "$WORKDIR" plan

echo "WinM installer scaffold downloaded to $WORKDIR"
echo "Review the plan before running any disk-modifying command."
