#!/usr/bin/env bash
if ! command -v emacs >/dev/null 2>&1; then
  vize-ci-apt-retry sudo apt-get update
  vize-ci-apt-retry sudo apt-get install -y emacs-nox
fi
