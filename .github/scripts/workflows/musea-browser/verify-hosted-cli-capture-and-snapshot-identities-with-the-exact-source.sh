#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/musea-browser/verify-hosted-cli-capture-and-snapshot-identities-with-the-exact-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec tsx --test gallery/components/hosted-vrt.browser.test.ts
vp exec tsx --test gallery/components/snapshot-identities.browser.test.ts
