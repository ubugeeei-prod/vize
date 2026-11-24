#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
mkdir -p "$OBSERVATION_ROOT"
profile=crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/overlay_profile.rs
cp "$profile" "$OBSERVATION_ROOT/overlay_profile.rs"
git checkout --detach "$BASE_SHA"
cp "$OBSERVATION_ROOT/overlay_profile.rs" "$profile"
if ! git show "$BASE_SHA:crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs" | rg -q '^mod overlay_profile;'; then
  printf '\n#[cfg(test)]\nmod overlay_profile;\n' >> crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs
fi
git rev-parse HEAD > "$OBSERVATION_ROOT/before-source.txt"
rustc -Vv > "$OBSERVATION_ROOT/before-rustc.txt"
git diff --binary > "$OBSERVATION_ROOT/before-instrumentation.patch"
sha256sum "$profile" > "$OBSERVATION_ROOT/before-instrumentation.sha256"
cargo test --locked --profile ci -p vize_canon --lib --no-run --message-format=json > "$OBSERVATION_ROOT/before-build.jsonl" 2> "$OBSERVATION_ROOT/before-build.stderr"
node "$VIZE_CI_FILES/capture-identical-test-only-instrumentation-at-the-exact-parent-source.ts"
executable=$(cat "$OBSERVATION_ROOT/before-executable.txt")
sha256sum "$executable" > "$OBSERVATION_ROOT/before-executable.sha256"
for round in 1 2 3; do
  VIZE_OVERLAY_PROFILE_OUTPUT="$OBSERVATION_ROOT/before-$round.json" "$executable" corsa_bridge::vue_dependencies_alias::context::cache::overlay_profile::profile_unrelated_overlays --exact --ignored --test-threads=1 > "$OBSERVATION_ROOT/before-$round.stdout" 2> "$OBSERVATION_ROOT/before-$round.stderr"
done
# Only this disposable Actions checkout contains the disclosed test
# injection. Restore it before returning to the literal candidate.
git restore crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs
if ! git cat-file -e "$BASE_SHA:$profile" 2>/dev/null; then rm "$profile"; else git restore "$profile"; fi
git checkout --detach "$SOURCE_SHA"
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
rustc -Vv > "$OBSERVATION_ROOT/after-rustc.txt"
test -z "$(git status --porcelain --untracked-files=no)"
