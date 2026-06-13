#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$HOME/.config/sgs"

cp -r "$SCRIPT_DIR"/. "$HOME/.config/sgs/"
