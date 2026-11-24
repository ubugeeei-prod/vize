#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/setup-rust-script/extract-prebuilt.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail

archive="${1:?expected archive}"
destination="${2:?expected fresh destination}"
digest="${3:?expected pinned digest}"
if [ "$#" -ne 3 ] || [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]] || \
  [ ! -f "$archive" ] || [ -L "$archive" ] || [ -e "$destination" ]; then
  echo "::error::invalid rust-script archive inputs" >&2
  exit 1
fi
actual_digest="$(sha256sum "$archive")"
if [ "${actual_digest%% *}" != "$digest" ]; then
  echo "::error::rust-script archive digest mismatch" >&2
  exit 1
fi

# Validate both complete membership and member type before writing any entry.
if [ "$(tar -tzf "$archive")" != "rust-script" ]; then
  echo "::error::expected only the rust-script archive member" >&2
  exit 1
fi
listing="$(tar -tvzf "$archive")"
if [[ "$listing" != -* || "$listing" == *$'\n'* ]]; then
  echo "::error::expected one regular rust-script file" >&2
  exit 1
fi
mkdir "$destination"
tar --no-same-owner --no-same-permissions -xzf "$archive" -C "$destination"
if [ ! -f "$destination/rust-script" ] || [ -L "$destination/rust-script" ]; then
  echo "::error::rust-script extraction is not a regular file" >&2
  exit 1
fi
