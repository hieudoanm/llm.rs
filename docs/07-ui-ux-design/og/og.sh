#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Create PNG from SVG only
if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w 1200 -h 630 svg/og.svg > images/og.png
elif command -v magick >/dev/null 2>&1; then
    magick svg/og.svg -background '#0f172a' images/og.png
else
    echo "rsvg-convert or magick required"
    exit 1
fi
