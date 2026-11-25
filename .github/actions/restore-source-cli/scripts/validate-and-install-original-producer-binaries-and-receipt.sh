#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/restore-source-cli/scripts/validate-and-install-original-producer-binaries-and-receipt.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/compat/github/source-cli-artifact.ts restore "$RUNNER_TEMP/source-cli/source-cli.tar.gz" "$SOURCE_CLI_SHA256" "$SOURCE_CLI_PRODUCER_ATTEMPT" "$SOURCE_CLI_RECEIPT_SHA256" && cat target/ci/vize.differential-build.json && sudo install -m 0755 target/ci/vize /usr/local/bin/vize
