#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-profile/scripts/build-the-native-binding-with-frame-pointers-and-the-vize-package.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
roles=(head)
if [[ -n "$BASE_SHA" ]]; then roles+=(base); fi
for role in "${roles[@]}"; do
  (
    cd "$RUNNER_TEMP/profile-$role"
    vp install --frozen-lockfile --prefer-offline --filter './npm/cli...' --filter './npm/native...'
    cargo --version
  )
done
export RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes"
export CARGO_PROFILE_CI_OPT_STRIP=none
pids=()
for role in "${roles[@]}"; do
  (
    cd "$RUNNER_TEMP/profile-$role/npm/native"
    rm -rf .artifacts/native
    VIZE_NATIVE_BUILD_TOKEN="$(openssl rand -hex 16)" CARGO_TARGET_DIR="$RUNNER_TEMP/profile-$role/target" \
      node_modules/.bin/napi build --platform --profile ci-opt \
        --manifest-path ../../crates/vize_vitrine/Cargo.toml -p vize_vitrine \
        --features napi,legacy --output-dir .artifacts/native --no-js > "$RUNNER_TEMP/build-$role.log" 2>&1
    cp .artifacts/native/*.node .
  ) &
  pids+=("$!")
done
status=0
for pid in "${pids[@]}"; do wait "$pid" || status=1; done
for role in "${roles[@]}"; do echo "::group::native build $role"; tail -n 40 "$RUNNER_TEMP/build-$role.log"; echo "::endgroup::"; done
[[ "$status" == 0 ]]
for role in "${roles[@]}"; do
  (cd "$RUNNER_TEMP/profile-$role" && vp run --filter './npm/cli' build)
  file "$RUNNER_TEMP/profile-$role"/npm/native/*.node
done
