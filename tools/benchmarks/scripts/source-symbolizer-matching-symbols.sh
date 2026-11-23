#!/usr/bin/env bash
set -euo pipefail
nm --defined-only --numeric-sort "$1" |
  awk -v symbol="$2" '$0 ~ (" [tT] .*" symbol) { print; found=1 } END { if (!found) exit 1 }'
