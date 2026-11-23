#!/usr/bin/env bash
node tools/commands/release/npm/smoke-wasm-package.ts npm/wasm
node tools/commands/release/npm/smoke-release-install.ts npm/wasm
