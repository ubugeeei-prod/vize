#!/usr/bin/env bash
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/instance-globals-10617/source-build.json"
sha256sum target/ci/vize > "$RUNNER_TEMP/instance-globals-10617/source-cli.sha256"
