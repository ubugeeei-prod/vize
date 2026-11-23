#!/usr/bin/env bash
set -euo pipefail
tar --exclude=node_modules -C "$RUNNER_TEMP" -czf "$RUNNER_TEMP/component-scope-binding-raw.tgz" component-scope-binding-evidence
