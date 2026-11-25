#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-lsp-vs-volar/scripts/stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Same profile as the vize rows; published addons use `release`.
cargo build --profile ci-opt -p vize_vitrine --features napi,legacy
root="$RUNNER_TEMP/tsserver-plugin/node_modules"
mkdir -p "$root/@vizejs"
cp -R npm/native "$root/@vizejs/native"
cp target/ci-opt/libvize_vitrine.so "$root/@vizejs/native/vize-vitrine.linux-x64-gnu.node"
cp -R npm/typescript-plugin "$root/@vizejs/typescript-vue-plugin"
rm -rf "$root/@vizejs/native/node_modules" "$root/@vizejs/typescript-vue-plugin/node_modules"
# The project's TypeScript: the same tsserver the Volar row runs.
ln -s "$RUNNER_TEMP/tools/node_modules/typescript" "$root/typescript"

if [ -n "$COMPARE_REF" ]; then
  root="$RUNNER_TEMP/tsserver-plugin-compare/node_modules"
  mkdir -p "$root/@vizejs"
  cp -R "$RUNNER_TEMP/compare/npm/native" "$root/@vizejs/native"
  cp "$RUNNER_TEMP/vize-vitrine-compare.node" "$root/@vizejs/native/vize-vitrine.linux-x64-gnu.node"
  cp -R "$RUNNER_TEMP/compare/npm/typescript-plugin" "$root/@vizejs/typescript-vue-plugin"
  rm -rf "$root/@vizejs/native/node_modules" "$root/@vizejs/typescript-vue-plugin/node_modules"
  ln -s "$RUNNER_TEMP/tools/node_modules/typescript" "$root/typescript"
fi
