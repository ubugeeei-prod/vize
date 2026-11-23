#!/usr/bin/env bash
vp run --no-cache --filter './npm/native' build:debug && vp run --workspace-root check:ci
