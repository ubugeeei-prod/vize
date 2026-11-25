#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-volar-plugin/scripts/install-exact-official-volar-and-typescript-service-providers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "VIZE_VOLAR_EVIDENCE_DIR=$RUNNER_TEMP/volar-plugin-evidence" >> "$GITHUB_ENV"
echo "VIZE_VOLAR_DEPENDENCY_LOCK=$RUNNER_TEMP/volar-plugin-deps/package-lock.json" >> "$GITHUB_ENV"
npm install --prefix "$RUNNER_TEMP/volar-plugin-deps" --ignore-scripts --no-audit --no-fund @volar/language-core@2.4.30 @volar/typescript@2.4.30 typescript@6.0.3 vue@3.5.42
mkdir -p "$RUNNER_TEMP/volar-plugin-deps/node_modules/@vizejs"
ln -s "$GITHUB_WORKSPACE/npm/native" "$RUNNER_TEMP/volar-plugin-deps/node_modules/@vizejs/native"
ln -s "$RUNNER_TEMP/volar-plugin-deps/node_modules" "$GITHUB_WORKSPACE/npm/volar/node_modules"
