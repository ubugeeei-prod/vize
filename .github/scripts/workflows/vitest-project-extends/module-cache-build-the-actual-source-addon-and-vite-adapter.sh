#!/usr/bin/env bash
vp run --workspace-root build:native:test
vp run --workspace-root build:vite-plugin
node npm/builder/vite/scripts/run-tests.ts --require-prepared
