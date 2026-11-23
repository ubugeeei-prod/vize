#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/slot-provider-any"
tar --exclude=project --exclude=node_modules --exclude=tmp --exclude=cache -czf "$RUNNER_TEMP/slot-provider-any.tgz" -C "$RUNNER_TEMP" slot-provider-any
