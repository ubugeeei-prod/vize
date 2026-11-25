#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/commands/ci/github/zig-musl/cc.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail

name=${0##*/}
target=${name#zig-cc-}
args=()
while (($#)); do
  case "$1" in
    --target=*) shift ;;
    --target)
      shift
      if (($#)); then shift; fi
      ;;
    *) args+=("$1"); shift ;;
  esac
done

exec zig cc -target "$target" "${args[@]}"
