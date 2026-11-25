#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/l1-document-special-text/compare-the-original-packet-with-chromium.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp node --test \
  tests/tooling/davinci-document-body-elements.test.ts \
  tests/tooling/davinci-document-disclosure-elements.test.ts \
  tests/tooling/davinci-document-embedded-fallback.test.ts \
  tests/tooling/davinci-document-foreign-content.test.ts \
  tests/tooling/davinci-document-phrasing.test.ts \
  tests/tooling/davinci-document-select.test.ts \
  tests/tooling/davinci-document-special-text.test.ts
