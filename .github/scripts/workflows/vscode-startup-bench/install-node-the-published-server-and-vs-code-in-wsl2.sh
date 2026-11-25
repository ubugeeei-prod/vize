#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vscode-startup-bench/install-node-the-published-server-and-vs-code-in-wsl2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
version=24.14.0
mkdir -p "$HOME/server"
cp ext-builds/server/vize "$HOME/server/vize" && chmod +x "$HOME/server/vize"
"$HOME/server/vize" --version
cd "$HOME"
curl -fsSLO "https://nodejs.org/dist/v$version/node-v$version-linux-x64.tar.xz"
curl -fsSLO "https://nodejs.org/dist/v$version/SHASUMS256.txt"
grep " node-v$version-linux-x64.tar.xz$" SHASUMS256.txt | sha256sum --check
tar -xJf "node-v$version-linux-x64.tar.xz" -C /usr/local --strip-components=1
mkdir -p "$HOME/te" && cd "$HOME/te"
npm install --no-audit --no-fund --ignore-scripts @vscode/test-electron@3.1.0 > /dev/null
node "$VIZE_CI_FILES/install-node-the-published-server-and-vs-code-in-wsl2.cjs"
cat "$HOME/code-path"
