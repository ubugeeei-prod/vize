#!/usr/bin/env bash
node tools/support/compat/github/source-cli-artifact.ts restore "$RUNNER_TEMP/source-cli/source-cli.tar.gz" "$SOURCE_CLI_SHA256" "$SOURCE_CLI_PRODUCER_ATTEMPT" "$SOURCE_CLI_RECEIPT_SHA256" && cat target/ci/vize.differential-build.json && sudo install -m 0755 target/ci/vize /usr/local/bin/vize
