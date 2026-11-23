#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/cold-scale"
tar --exclude='*/project' --exclude='*/xdg-cache' --exclude='*/tmp' -czf "$RUNNER_TEMP/cold-scale-evidence.tgz" -C "$RUNNER_TEMP" cold-scale
