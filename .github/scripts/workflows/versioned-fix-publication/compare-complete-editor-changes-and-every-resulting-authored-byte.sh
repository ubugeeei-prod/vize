#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
suite=(-p vize_maestro --features experimental-versioned-fixes)
cargo clippy "${flags[@]}" -p vize_l0 --lib --features positioned-edits --no-deps -- -D warnings
cargo clippy "${flags[@]}" -p vize_l1 --lib --features positioned-edits --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${suite[@]}" --test versioned_fix_publication --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_l0 --lib --features positioned-edits line_index::
cargo test "${flags[@]}" -p vize_l1 --lib --features positioned-edits,diagnostic-edit-selection edit::
cargo test "${flags[@]}" "${suite[@]}" --lib source_project::edit_tests
cargo test "${flags[@]}" "${suite[@]}" --test versioned_fix_publication
export VIZE_VERSIONED_FIX_INPUTS="$RUNNER_TEMP/versioned-fixture-paths"
git ls-files -z -- '*.vue' '*.vue.txt' '*.html' '*.html.txt' '*.tsx' '*.tsx.txt' '*.jsx' '*.jsx.txt' > "$VIZE_VERSIONED_FIX_INPUTS"
VIZE_VERSIONED_FIX_TIMINGS=1 cargo test "${flags[@]}" "${suite[@]}" --test versioned_fix_publication original_lint_fixes -- --exact --ignored --nocapture
cargo test "${flags[@]}" "${suite[@]}" --test versioned_fix_publication profile::publication_cost -- --exact --ignored --nocapture
evidence="$RUNNER_TEMP/versioned-edit-times"
mkdir -p "$evidence"
binary=$(cargo test "${flags[@]}" "${suite[@]}" --test versioned_fix_publication --no-run --message-format=json | jq -ers '[.[] | select(.reason == "compiler-artifact" and .target.name == "versioned_fix_publication")] | last | .executable')
# Each process measures the two entries in counterbalanced pairs.
for round in 0 1 2 3 4 5; do
  for query in original_lint_fixes profile::publication_cost; do
    VIZE_VERSIONED_FIX_TIMINGS=1 "$binary" "$query" --exact --ignored --nocapture > "$evidence/$round-${query##*::}.log" 2>&1
  done
done
