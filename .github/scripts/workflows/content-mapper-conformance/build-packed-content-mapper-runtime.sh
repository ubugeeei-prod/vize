#!/usr/bin/env bash
vp run --filter './npm/native' build:ci
(cd npm/cli && vp pack)
