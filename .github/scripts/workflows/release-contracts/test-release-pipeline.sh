#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-contracts/test-release-pipeline.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node --test \
  tests/tooling/release/npm-resume.test.ts \
  tests/tooling/release/public-provider.test.ts \
  tests/tooling/release/release-workflow.test.ts \
  tests/tooling/release/release-notes.test.ts \
  tests/tooling/release/verify-published-registry.test.ts \
  tests/tooling/release/release-signing.test.ts \
  tests/tooling/release/editor-platforms.test.ts \
  tests/tooling/github-workflows-release-build.test.ts \
  tests/tooling/release-jsr-workflow.test.ts \
  tests/tooling/jsr-fresh-provenance.test.ts \
  tests/tooling/drop-in-claim-scope.test.ts \
  tests/tooling/package-manifests.test.ts \
  tests/tooling/bundler-plugin-manifests.test.ts \
  tests/tooling/davinci-plugin-sdk-release.test.ts \
  tests/tooling/entry-packages.test.ts
