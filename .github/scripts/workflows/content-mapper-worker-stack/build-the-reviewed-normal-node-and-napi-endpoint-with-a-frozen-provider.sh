#!/usr/bin/env bash
set -euo pipefail
within_budget() {
  remaining="$(( (ADMISSION_DEADLINE_MS - $(date -u +%s) * 1000) / 1000 ))"
  test "$remaining" -gt 0
  timeout --signal=TERM --kill-after=5s "${remaining}s" "$@"
}
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/mapper-admission/source.txt"
node --version > "$RUNNER_TEMP/mapper-admission/toolchain.txt"
rustc -Vv >> "$RUNNER_TEMP/mapper-admission/toolchain.txt"
cargo -V >> "$RUNNER_TEMP/mapper-admission/toolchain.txt"
vp --version >> "$RUNNER_TEMP/mapper-admission/toolchain.txt"
mkdir "$RUNNER_TEMP/mapper-admission/controls"
ADMISSION_RETAIN_CONTROLS=1 TMPDIR="$RUNNER_TEMP/mapper-admission/controls" within_budget timeout --signal=TERM --kill-after=5s 120s node --test tests/tooling/content-mapper-admission-hosts*.test.ts > "$RUNNER_TEMP/mapper-admission/pure.log" 2>&1
within_budget vp install --frozen-lockfile --prefer-offline > "$RUNNER_TEMP/mapper-admission/source-install.log" 2>&1
provider_fixture=tests/_fixtures/tooling/content-mapper-admission-provider
mkdir "$RUNNER_TEMP/admission-provider"
cp "$provider_fixture/package.json.txt" "$RUNNER_TEMP/admission-provider/package.json"
cp "$provider_fixture/package-lock.json.txt" "$RUNNER_TEMP/admission-provider/package-lock.json"
cp "$provider_fixture/provenance.json" "$RUNNER_TEMP/admission-provider/provenance.json"
cp -R "$provider_fixture" "$RUNNER_TEMP/mapper-admission/public-provider-inputs"
(cd "$RUNNER_TEMP/admission-provider" && within_budget npm ci --ignore-scripts --no-audit --no-fund --prefer-offline) > "$RUNNER_TEMP/mapper-admission/provider-install.log" 2>&1
cmp "$provider_fixture/package-lock.json.txt" "$RUNNER_TEMP/admission-provider/package-lock.json"
token="$(openssl rand -hex 16)"
printf 'ADMISSION_NATIVE_TOKEN=%s\n' "$token" >> "$GITHUB_ENV"
within_budget cargo clean --profile ci -p vize -p vize_vitrine
VIZE_NATIVE_BUILD_TOKEN="$token" within_budget cargo build --locked --profile ci -p vize_vitrine --features napi > "$RUNNER_TEMP/mapper-admission/native-build.log" 2>&1
cp target/ci/libvize_vitrine.so npm/native/vize-vitrine.linux-x64-gnu.node
(cd npm/cli && within_budget vp pack) > "$RUNNER_TEMP/mapper-admission/cli-pack.log" 2>&1
git diff --exit-code HEAD
sha256sum npm/native/vize-vitrine.linux-x64-gnu.node > "$RUNNER_TEMP/mapper-admission/native.sha256"
