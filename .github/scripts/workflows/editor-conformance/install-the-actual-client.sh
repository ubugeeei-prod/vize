#!/usr/bin/env bash
set -euo pipefail
if [ "$CLIENT" = vim ]; then
  package=vim-nox
else
  package=emacs-nox
fi
vize-ci-apt-retry sudo apt-get update
vize-ci-apt-retry sudo apt-get install -y "$package"
