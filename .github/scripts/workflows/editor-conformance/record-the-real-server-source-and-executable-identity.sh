#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/editor-conformance/record-the-real-server-source-and-executable-identity.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test "$(git rev-parse HEAD)" = "${GITHUB_SHA}"
{
  echo "producer=$(git rev-parse HEAD)"
  echo 'recipe=cargo build --profile ci -p vize'
  rustc --version --verbose
  cargo --version
  sha256sum rust-toolchain.toml Cargo.lock target/ci/vize
} > target/ci/vize-producer.txt
(cd target/ci && sha256sum vize > vize.sha256)
