#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-browser-oracles/scripts/special-text-compare-the-original-packet-with-chromium.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp node --test \
  tests/tooling/davinci-document-body-elements.test.ts \
  tests/tooling/davinci-document-disclosure-elements.test.ts \
  tests/tooling/davinci-document-embedded-fallback.test.ts \
  tests/tooling/davinci-document-foreign-content.test.ts \
  tests/tooling/davinci-document-phrasing.test.ts \
  tests/tooling/davinci-document-select.test.ts \
  tests/tooling/davinci-document-special-text.test.ts
