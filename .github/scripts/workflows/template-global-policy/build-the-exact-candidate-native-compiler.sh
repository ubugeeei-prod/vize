#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$VIZE_TEMPLATE_GLOBAL_SOURCE_SHA"
token="$(openssl rand -hex 16)"
VIZE_NATIVE_BUILD_TOKEN="$token" cargo build --locked --profile ci -p vize_vitrine --features napi
cp target/ci/libvize_vitrine.so npm/native/vize-vitrine.linux-x64-gnu.node
echo "VIZE_TEMPLATE_GLOBAL_EXPECTED_TOKEN=$token" >> "$GITHUB_ENV"
