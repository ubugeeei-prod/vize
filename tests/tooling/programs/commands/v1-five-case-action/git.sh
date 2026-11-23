#!/usr/bin/env bash
set -euo pipefail
test "$1" = rev-parse
test "$2" = HEAD
printf "%s\n" "$SOURCE_SHA"
