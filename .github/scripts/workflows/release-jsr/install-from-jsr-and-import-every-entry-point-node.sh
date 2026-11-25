#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/install-from-jsr-and-import-every-entry-point-node.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
npm install vite@8 vue@3
npx --yes jsr add "@vizejs/vize@${JSR_VERSION#v}"
node --experimental-strip-types consumer.ts @vizejs/vize --vite
