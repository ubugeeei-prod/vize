#!/usr/bin/env bash
set -euo pipefail
. /etc/os-release
echo "container: ${PRETTY_NAME}; $(ldd --version 2>&1 | head -n 1)"
mode=$1
platform=$2
if [[ "$mode" == cli ]]; then
  binary="/work/$3"
  "$binary" --version
  mkdir -p /tmp/vize-cli-smoke
  printf '%s\n' '<template><div>{{ msg }}</div></template>' '<script setup lang="ts">' 'const msg = "ok";' '</script>' > /tmp/vize-cli-smoke/App.vue
  exec "$binary" lint /tmp/vize-cli-smoke/App.vue --no-config
fi

export PATH=/opt/vize-node/bin:/tmp/vize-global/bin:$PATH npm_config_prefix=/tmp/vize-global
echo "node $(node --version)"
if [[ "$mode" == binding ]]; then
  exec node /work/tools/support/compat/npm/glibc-floor-binding.cjs "/work/npm/native/npm/$platform/vize-vitrine.$platform.node"
fi

mkdir -p /tmp/vize-global/bin
corepack enable --install-directory /tmp/vize-global/bin
npm install --global --allow-scripts=bun bun@1.3.14 vite-plus@0.1.24
exec node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks npm/native npm/native/npm/* npm/cli npm/builder/unplugin npm/builder/vite
