#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/install-the-actual-client.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ "$CLIENT" = vim ]; then
  package=vim-nox
else
  package=emacs-nox
fi
vize-ci-apt-retry sudo apt-get update
vize-ci-apt-retry sudo apt-get install -y "$package"
