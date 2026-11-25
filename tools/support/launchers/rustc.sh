#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/support/launchers/rustc.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Cargo runs: rustc.sh <rustc or clippy-driver> [rustc] <arguments>
case "${VIZE_RUSTC_FRONTEND_JOBS:-1}" in
  '' | 0 | 1) ;;
  *)
    workspace="${0%/tools/support/launchers/rustc.sh}"
    case "${CARGO_MANIFEST_DIR:-}" in
      "$workspace"/*)
        previous=
        for argument in "$@"; do
          if [ "$previous" = --crate-name ]; then
            if [ "$argument" = "${CARGO_CRATE_NAME:-}" ]; then
              RUSTC_BOOTSTRAP="$argument"
              export RUSTC_BOOTSTRAP
              set -- "$@" -Zunstable-options "--jobs-frontend=$VIZE_RUSTC_FRONTEND_JOBS"
            fi
            break
          fi
          previous="$argument"
        done
        ;;
    esac
    ;;
esac
if [ -n "${VIZE_KACHE:-}" ]; then
  exec "$VIZE_KACHE" "$@"
fi
exec "$@"
