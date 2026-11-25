#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/11-docs-build/scripts/install-japanese-rendering-fonts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
sudo apt-get update
sudo apt-get install -y --no-install-recommends fonts-noto-cjk
fc-match --format '%{family}\n' ':lang=ja'
