#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt-style-build/retain-the-exact-compared-javascript-adapters.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
adapters="$RUNNER_TEMP/$PROOF/compared-adapters"
mkdir -p "$adapters/candidate" "$adapters/published"
for adapter in vite-plugin nuxt; do
  published="$FIXTURE/node_modules/@vizejs/$adapter/dist"
  if [[ -d "$published" ]]; then cp -R "$published" "$adapters/published/$adapter"; fi
done
if [[ -d npm/builder/vite/dist ]]; then cp -R npm/builder/vite/dist "$adapters/candidate/vite-plugin"; fi
if [[ -d npm/framework/nuxt/dist ]]; then cp -R npm/framework/nuxt/dist "$adapters/candidate/nuxt"; fi
