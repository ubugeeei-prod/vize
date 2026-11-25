#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-source-checks/verify-configured-native-runtime-toolchain.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
# Verify the repository toolchain setup here, before a Cargo proxy can
# implicitly download missing components inside a runtime test step.
host=$(rustup run "$VIZE_NATIVE_RUST_TOOLCHAIN" rustc --version --verbose | sed -n -f "$VIZE_CI_FILES/../shared/rust-host-triple.sed")
test -n "$host"
components=$(rustup component list --toolchain "$VIZE_NATIVE_RUST_TOOLCHAIN" --installed)
targets=$(rustup target list --toolchain "$VIZE_NATIVE_RUST_TOOLCHAIN" --installed)
printf '%s\n' "$components" "$targets"
IFS=',' read -r -a required_components <<< "$VIZE_NATIVE_RUST_COMPONENTS"
for component in "${required_components[@]}"; do
  if ! grep -Fxq -- "$component" <<< "$components" && ! grep -Fxq -- "$component-$host" <<< "$components"; then
    echo "::error::Missing configured Rust component: $component" >&2
    exit 1
  fi
done
IFS=',' read -r -a required_targets <<< "$VIZE_NATIVE_RUST_TARGETS"
for target in "${required_targets[@]}"; do
  if ! grep -Fxq -- "$target" <<< "$targets"; then
    echo "::error::Missing configured Rust target: $target" >&2
    exit 1
  fi
done
