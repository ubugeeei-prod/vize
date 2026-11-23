#!/usr/bin/env bash
vp run --filter './npm/cli' build
vp run --filter './npm/builder/unplugin' build
vp run --filter './npm/builder/vite' build
