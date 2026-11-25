#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/install-from-jsr-and-import-every-entry-point-bun.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
bun add vite@8 vue@3
bunx jsr add "@vizejs/vize@${JSR_VERSION#v}"
bun run consumer.ts @vizejs/vize --vite
