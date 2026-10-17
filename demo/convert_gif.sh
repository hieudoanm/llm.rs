#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

if [ ! -f demo.mp4 ]; then
  echo "demo.mp4 not found"
  exit 1
fi

if command -v ffmpeg >/dev/null 2>&1; then
  ffmpeg -y -i demo.mp4 -vf "fps=10,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" demo.gif
  echo "Created demo.gif"
else
  echo "ffmpeg not found"
  exit 1
fi
