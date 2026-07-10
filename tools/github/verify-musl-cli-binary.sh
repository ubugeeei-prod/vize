#!/usr/bin/env bash
set -euo pipefail

target="${1:?Usage: verify-musl-cli-binary.sh <rust-target>}"
binary="target/$target/release/vize"

file "$binary"

if readelf -l "$binary" | grep -q 'Requesting program interpreter'; then
  echo "::error ::musl CLI binary has a dynamic interpreter"
  readelf -l "$binary"
  exit 1
fi

if strings "$binary" | grep -Eq 'GLIBC_[0-9]'; then
  echo "::error ::musl CLI binary contains glibc version requirements"
  strings "$binary" | grep -E 'GLIBC_[0-9]' | sort -u
  exit 1
fi
