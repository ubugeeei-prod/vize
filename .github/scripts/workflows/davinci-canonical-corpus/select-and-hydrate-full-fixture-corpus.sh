#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-canonical-corpus/select-and-hydrate-full-fixture-corpus.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -o pipefail
node tools/support/compat/fixtures/davinci-dom-corpus-workflow.ts hydrate 2>&1 | tee real-project-davinci-dom-corpus/initial-hydration.log
node tools/support/compat/github/canonical-corpus-observer.mjs rehydrate
