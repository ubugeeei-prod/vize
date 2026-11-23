#!/usr/bin/env bash
vp run --filter './npm/native' build:ci
vp run --filter './npm/linter/oxlint' build
vp run --filter './npm/linter/eslint' build
