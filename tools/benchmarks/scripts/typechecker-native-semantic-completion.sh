#!/usr/bin/env bash
# Whole original parser handoff and observed native teardown, on the existing target.
set -euo pipefail
# The supplemental CLI/owner law qualifies PR source; manual phase inputs stay frozen.
if [[ "${GITHUB_EVENT_NAME:-}" == workflow_dispatch ]]; then exit 0; fi
export VIZE_TEST_REQUIRE_TSGO=1
if [[ -z "${VIZE_TEST_TSGO_PATH:-}" ]]; then
  export VIZE_TEST_TSGO_PATH="$(node --input-type=module <<'JS'
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { readFileSync, realpathSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
const require = createRequire(join(process.cwd(), 'package.json'));
const name = `@typescript/typescript-${process.platform}-${process.arch}`;
const manifest = realpathSync(require.resolve(`${name}/package.json`));
const metadata = JSON.parse(readFileSync(manifest, 'utf8'));
assert.equal(metadata.name, name); assert.equal(metadata.version, '7.0.2');
const native = realpathSync(join(dirname(manifest), 'lib/tsc'));
const probe = spawnSync(native, ['--version'], { encoding: 'utf8', timeout: 15000 });
assert.equal(probe.error, undefined); assert.equal(probe.status, 0);
assert.equal(probe.stdout, 'Version 7.0.2\n'); assert.equal(probe.stderr, '');
process.stdout.write(native);
JS
)"
fi
[[ "${VIZE_TEST_REQUIRE_TSGO:-}" == 1 ]]
[[ -n "${SOURCE_SHA:-}" && -n "${RUNNER_TEMP:-}" ]]
[[ "$(git rev-parse HEAD)" == "$SOURCE_SHA" ]]
[[ -x "${VIZE_TEST_TSGO_PATH:-}" ]]
unset VIZE_TEST_DISABLE_TSGO VIZE_NESTED_BATCH_CAPTURE_DIR
export CORSA_PATH="$VIZE_TEST_TSGO_PATH"
export VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR="$RUNNER_TEMP/native-source-recipe-semantic-completion"
mkdir "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR"
node tests/_fixtures/corsa-process-receipt/verify-vendor.mjs > "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/vendor-custody.json"
printf '%s\n' "$SOURCE_SHA" > "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/source-sha.txt"
options=(--locked --profile ci-opt --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16')
# The already source-built libtest and CLI targets share profile, features and cache.
factory_status=0
cargo test "${options[@]}" -p vize_canon --lib native_semantic_completion -- --nocapture --test-threads=1 \
  > >(tee "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/factory.stdout.bin") \
  2> >(tee "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/factory.stderr.bin" >&2) || factory_status=$?
cli_status=0
cargo test "${options[@]}" -p vize --test check_canon_native_completion_cli --config 'profile.ci-opt.package.vize.strip="symbols"' -- --nocapture --test-threads=1 \
  > >(tee "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/cli-test.stdout.bin") \
  2> >(tee "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/cli-test.stderr.bin" >&2) || cli_status=$?
wait
printf 'factory=%s\ncli=%s\n' "$factory_status" "$cli_status" > "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/test-status.txt"
[[ "$factory_status" == 0 && "$cli_status" == 0 ]]
rg -q -F 'test result: ok. 1 passed; 0 failed;' "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/factory.stdout.bin"
rg -q -F 'test result: ok. 1 passed; 0 failed;' "$VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR/cli-test.stdout.bin"
