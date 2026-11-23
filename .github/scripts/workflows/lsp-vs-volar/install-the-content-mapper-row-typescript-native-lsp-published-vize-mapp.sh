#!/usr/bin/env bash
mkdir "$RUNNER_TEMP/mapper" && cd "$RUNNER_TEMP/mapper"
echo '{"private":true}' > package.json
npm install --no-audit --no-fund "@typescript/typescript-linux-x64@$TSGO_MAPPER" "vize@$MAPPER_VIZE"
node "$VIZE_CI_FILES/install-the-content-mapper-row-typescript-native-lsp-published-vize-mapp.cjs" | tee versions.txt
