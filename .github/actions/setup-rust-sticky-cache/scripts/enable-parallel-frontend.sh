#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/setup-rust-sticky-cache/scripts/enable-parallel-frontend.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$VIZE_RUSTC_FRONTEND_JOBS" =~ ^[1-9][0-9]*$ ]]
wrapper="$GITHUB_WORKSPACE/tools/support/launchers/rustc.sh"
# A job that set its own wrapper, or checked out a tree without this one,
# keeps its compiler invocation unchanged.
if [[ -z "${RUSTC_WRAPPER:-}" && -x "$wrapper" ]]; then
  {
    printf 'RUSTC_WRAPPER=%s\n' "$wrapper"
    printf 'VIZE_RUSTC_FRONTEND_JOBS=%s\n' "$VIZE_RUSTC_FRONTEND_JOBS"
  } >> "$GITHUB_ENV"
fi
