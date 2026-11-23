#!/usr/bin/env bash
vp run --workspace-root build:native:test
vp run --filter './npm/linter/oxlint' build
