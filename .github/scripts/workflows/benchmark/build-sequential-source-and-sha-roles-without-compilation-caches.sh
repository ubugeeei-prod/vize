#!/usr/bin/env bash
set -euo pipefail
mkdir check-message-build
compiler="$(rustup which rustc)"
cargo_binary="$(rustup which cargo)"
"$compiler" -vV > check-message-build/rustc.txt
"$cargo_binary" -V > check-message-build/cargo.txt
sha256sum "$compiler" "$cargo_binary" > check-message-build/toolchain.sha256
for role in base head; do
  sha="$(git -C "$role" rev-parse HEAD)"
  target="$RUNNER_TEMP/check-message-$role-$sha-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  [[ ! -e "$target" && ! -L "$target" ]]
  printf '{"role":"%s","sourceSha":"%s","target":"%s","initialAbsent":true,"compilationCache":false,"profile":"ci-opt-release-thin-lto-cgu16"}\n' "$role" "$sha" "$target" > "check-message-build/$role.json"
  git -C "$role" status --porcelain > "check-message-build/$role-status-before.txt"
  [[ ! -s "check-message-build/$role-status-before.txt" ]]
  env -u CARGO_TARGET_DIR -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS \
    RUSTC="$compiler" RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= \
    "$cargo_binary" build --manifest-path "$role/Cargo.toml" --locked --profile ci-opt -p vize --target-dir "$target" \
    --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' \
    > "check-message-build/$role.stdout" 2> "check-message-build/$role.stderr"
  git -C "$role" diff --exit-code
  sha256sum "$target/ci-opt/vize" > "check-message-build/$role-binary.sha256"
  printf 'MESSAGE_%s_BIN=%s\n' "${role^^}" "$target/ci-opt/vize" >> "$GITHUB_ENV"
done
