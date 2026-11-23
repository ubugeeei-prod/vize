#!/usr/bin/env bash
vp run --filter './npm/native' build:ci && vp run --workspace-root build:packages
