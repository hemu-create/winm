#!/usr/bin/env bash
set -euo pipefail

WORKDIR="${WINM_WORKDIR:-$HOME/winm-work}"
mkdir -p "$WORKDIR"

INSTALLER="$WORKDIR/winm_install.py"
CDN_URL="https://cdn.jsdelivr.net/gh/hemu-create/winm@master/installer/winm_install.py"
RAW_URL="https://raw.githubusercontent.com/hemu-create/winm/master/installer/winm_install.py"

download() {
    curl --fail --silent --show-error --location \
        --connect-timeout 5 --max-time 30 \
        --proto '=https' --tlsv1.2 "$1" -o "$INSTALLER"
}

if ! download "$CDN_URL"; then
    echo "CDN download failed; trying raw GitHub..." >&2
    if ! download "$RAW_URL"; then
        echo "curl failed; retrying with Python urllib..." >&2
        python3 - "$RAW_URL" "$INSTALLER" <<'PY'
import sys
import urllib.request

url, output = sys.argv[1], sys.argv[2]
with urllib.request.urlopen(url, timeout=30) as response, open(output, "wb") as handle:
    handle.write(response.read())
PY
    fi
fi

cat <<'GUIDE'
WinM installer
==============

This follows the Asahi installer flow:

1. Check macOS/Apple Silicon and required tools.
2. List disks and partitions.
3. Plan the APFS resize and ESP/NTFS partitions.
4. Confirm before any destructive command.
5. Download the WinM b00t image.
6. Show the remaining Windows media/boot-object steps.

Keep a full backup and a second Mac or Apple Configurator for DFU recovery.
GUIDE

echo
if [ -r /dev/tty ]; then
    exec python3 "$INSTALLER" --workdir "$WORKDIR" interactive </dev/tty
fi

echo "No interactive terminal found; running non-destructive checks only."
python3 "$INSTALLER" --workdir "$WORKDIR" check
python3 "$INSTALLER" --workdir "$WORKDIR" guide