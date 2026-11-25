#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-vs-volar/install-the-content-mapper-row-typescript-native-lsp-published-vize-mapp.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir "$RUNNER_TEMP/mapper" && cd "$RUNNER_TEMP/mapper"
echo '{"private":true}' > package.json
npm install --no-audit --no-fund "@typescript/typescript-linux-x64@$TSGO_MAPPER" "vize@$MAPPER_VIZE"
node "$VIZE_CI_FILES/install-the-content-mapper-row-typescript-native-lsp-published-vize-mapp.cjs" | tee versions.txt
