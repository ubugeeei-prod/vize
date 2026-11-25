#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/install-node-24-in-wsl2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
version=24.14.0
curl -fsSLO "https://nodejs.org/dist/v$version/node-v$version-linux-x64.tar.xz"
curl -fsSLO "https://nodejs.org/dist/v$version/SHASUMS256.txt"
grep " node-v$version-linux-x64.tar.xz$" SHASUMS256.txt | sha256sum --check
tar -xJf "node-v$version-linux-x64.tar.xz" -C /usr/local --strip-components=1
node --version
uname -a
