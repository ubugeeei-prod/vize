#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-volar-plugin/scripts/build-the-actual-reviewed-native-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$VIZE_SOURCE_SHA"
token="$(openssl rand -hex 16)"
VIZE_NATIVE_BUILD_TOKEN="$token" cargo build --profile ci -p vize_vitrine --features napi
cp target/ci/libvize_vitrine.so npm/native/vize-vitrine.linux-x64-gnu.node
echo "VIZE_VOLAR_EXPECTED_NATIVE_TOKEN=$token" >> "$GITHUB_ENV"
echo "NAPI_RS_NATIVE_LIBRARY_PATH=$GITHUB_WORKSPACE/npm/native/vize-vitrine.linux-x64-gnu.node" >> "$GITHUB_ENV"
