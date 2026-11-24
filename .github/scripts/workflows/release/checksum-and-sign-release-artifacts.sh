#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/checksum-and-sign-release-artifacts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
shopt -s nullglob
for artifact in cli-artifacts/*.tar.gz cli-artifacts/*.zip editor-extension-artifacts/editors/vscode/dist/*.vsix editor-extension-artifacts/platform/*.vsix editor-extension-artifacts/platform/*.receipt.json editor-extension-artifacts/*.tar.gz; do
  ( cd "$(dirname "$artifact")" && sha256sum "$(basename "$artifact")" ) > "$artifact.sha256"
done
node tools/support/release/sign_release_artifacts.ts "$RELEASE_TAG"
