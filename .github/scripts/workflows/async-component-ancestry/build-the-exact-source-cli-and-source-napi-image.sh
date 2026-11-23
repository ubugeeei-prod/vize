#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
token="$(openssl rand -hex 16)"
set +e
VIZE_NATIVE_BUILD_TOKEN="$token" cargo build --locked --profile ci -p vize_vitrine --features napi >target/vize-tests/async-component/napi-build.stdout 2>target/vize-tests/async-component/napi-build.stderr
napi_status=$?
printf '%s\n' "$napi_status" >target/vize-tests/async-component/napi-build.status
set -e
cat target/vize-tests/async-component/napi-build.stderr >&2
test "$napi_status" = 0
set +e
cargo build --locked --profile ci -p vize --bin vize >target/vize-tests/async-component/cli-build.stdout 2>target/vize-tests/async-component/cli-build.stderr
cli_status=$?
printf '%s\n' "$cli_status" >target/vize-tests/async-component/cli-build.status
set -e
cat target/vize-tests/async-component/cli-build.stderr >&2
test "$cli_status" = 0
cp target/ci/libvize_vitrine.so npm/native/vize-vitrine.linux-x64-gnu.node
echo "EXPECTED_TOKEN=$token" >> "$GITHUB_ENV"
