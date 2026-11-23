#!/usr/bin/env bash
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/router-editor-artifact/producer-build.json"
cp target/ci/vize.source-build.json "$RUNNER_TEMP/router-editor-artifact/producer-transport.json"
cp target/ci/vize.source-use.json "$RUNNER_TEMP/router-editor-artifact/consumer-use.json"
