#!/usr/bin/env bash
set -o pipefail
node tools/support/compat/fixtures/davinci-dom-corpus-workflow.ts hydrate 2>&1 | tee real-project-davinci-dom-corpus/initial-hydration.log
node tools/support/compat/github/canonical-corpus-observer.mjs rehydrate
