#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/github-zig-musl-linkers/readelf.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$1" == "--version-info" ]]; then
  if [[ "${VERIFY_MODE:-ok}" == "glibc" ]]; then
    printf 'GLIBC_2.39\n'
  else
    printf 'No version information found in this file.\n'
  fi
elif [[ "${VERIFY_MODE:-ok}" == "interpreter" ]]; then
  printf '[Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]\n'
else
  printf 'Program Headers:\n'
fi
