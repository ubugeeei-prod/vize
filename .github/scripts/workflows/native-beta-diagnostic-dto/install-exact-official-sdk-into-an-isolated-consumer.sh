#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/native-beta-diagnostic-dto/install-exact-official-sdk-into-an-isolated-consumer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
sdk_root="$RUNNER_TEMP/native-dto-sdk-$SDK_VERSION"
npm install --prefix "$sdk_root" --ignore-scripts --no-audit --no-fund "typescript@$SDK_VERSION"
echo "SDK_ROOT=$sdk_root" >> "$GITHUB_ENV"
command -v strace
