#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/publish-native-platforms-before-dependent-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
read -ra js <<< "$RELEASE_JS_PACKAGES"
platforms=(npm/native/npm)
native=(npm/native)
dependents=(npm/fresco-native npm/wasm "${js[@]}")
node tools/support/release/publish_npm_package_dirs/main.ts "${platforms[@]}" --concurrency 8 --provenance
node tools/support/release/publish_npm_package_dirs/main.ts "${native[@]}" --concurrency 1 --provenance
node tools/support/release/publish_npm_package_dirs/main.ts "${dependents[@]}" --concurrency 16 --provenance --no-wait
echo "packages=$(node tools/support/release/publish_npm_package_dirs/main.ts --names "${platforms[@]}" "${native[@]}" "${dependents[@]}")" >> "$GITHUB_OUTPUT"
