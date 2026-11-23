#!/usr/bin/env bash
set -euo pipefail
artifacts="$RUNNER_TEMP/worker-admission-head-$BUILD_PROFILE.jsonl"
evidence="$RUNNER_TEMP/worker-spawn-$BUILD_PROFILE.log"
cargo test --locked --profile "$BUILD_PROFILE" -p vize --lib --no-run --message-format=json > "$artifacts"
test_image="$(jq -r 'select(.reason == "compiler-artifact" and .target.name == "vize" and .profile.test == true and .executable != null) | .executable' "$artifacts")"
test -x "$test_image"
image_hash="$(sha256sum "$test_image" | cut -d' ' -f1)"
printf 'source=%s tree=%s dispatcher=%s tests=%s image=%s sha256=%s\n' "$SOURCE_SHA" "$(git rev-parse HEAD^{tree})" "$(git hash-object crates/vize/src/commands/content_mapper/dispatch.rs)" "$(git hash-object crates/vize/src/commands/content_mapper/dispatch/tests.rs)" "$test_image" "$image_hash" > "$evidence"
"$test_image" commands::content_mapper::dispatch::tests:: --nocapture | tee -a "$evidence"
test "$(sha256sum "$test_image" | cut -d' ' -f1)" = "$image_hash"
rg -F 'test result: ok. 7 passed; 0 failed;' "$RUNNER_TEMP/worker-spawn-$BUILD_PROFILE.log"
