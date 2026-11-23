#!/usr/bin/env bash
set -euo pipefail
directory="$RUNNER_TEMP/jsx-authored-native/vdom-child-values"
mkdir -p "$directory"
test "$(git rev-parse HEAD)" = "$QUALIFICATION_HEAD_SHA"
git rev-parse HEAD HEAD^{tree} > "$directory/head-source.txt"
baseline="$RUNNER_TEMP/jsx-vdom-values-base"
observer=crates/vize_atelier_jsx/tests/vdom_child_values.rs
git worktree add --detach "$baseline" 9c594186bae97cbc4b4c5441d9165b3354ccbec0
cp "$observer" "$baseline/$observer"
cmp "$observer" "$baseline/$observer"
git -C "$baseline" diff --exit-code -- crates/*/src davinci/*/src
git -C "$baseline" rev-parse HEAD HEAD^{tree} > "$directory/base-source.txt"
sha256sum "$observer" "$baseline/$observer" tests/tooling/jsx-vapor-oracle/{child-values.ts,vdom-child-values.ts,runtime.ts} tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json > "$directory/observer.sha256"
status=0
cargo test --locked --profile ci -p vize_l2 --test op_family --test jsx_child_dump > "$directory/typed-ir.stdout" 2> "$directory/typed-ir.stderr" || status=1
cargo test --locked --profile ci -p vize_l2_to_l3 --test jsx_child_value > "$directory/child-graph.stdout" 2> "$directory/child-graph.stderr" || status=1
cargo test --locked --profile ci -p vize_l3 --test op_family --test values_dump > "$directory/child-graph-codec.stdout" 2> "$directory/child-graph-codec.stderr" || status=1
cargo test --locked --profile ci -p vize_canon --test jsx_child_expression_control > "$directory/projection-control.stdout" 2> "$directory/projection-control.stderr" || status=1
lint_selector=jsx_child_expression_control::ordinary_template_and_v_text_keep_the_complete_undefined_reference_diagnostic
cargo test --locked --profile ci -p vize_patina --test integration "$lint_selector" -- --exact --list > "$directory/linter-control.list.stdout" 2> "$directory/linter-control.list.stderr" || status=1
cargo test --locked --profile ci -p vize_patina --test integration "$lint_selector" -- --exact > "$directory/linter-control.stdout" 2> "$directory/linter-control.stderr" || status=1
LINTER_CONTROL_DIRECTORY="$directory" LINTER_CONTROL_SELECTOR="$lint_selector" node "$VIZE_CI_FILES/verify-linter-control.ts" || status=1
VIZE_JSX_VDOM_CHILD_CAPTURE="$directory/head" cargo test --locked --profile ci -p vize_atelier_jsx --test vdom_child_values -- --nocapture > "$directory/head.stdout" 2> "$directory/head.stderr" || status=1
(cd "$baseline" && CARGO_TARGET_DIR="$RUNNER_TEMP/jsx-vdom-values-base-target" VIZE_JSX_VALUE_ORACLE_ROOT="$GITHUB_WORKSPACE" VIZE_JSX_VDOM_EXPECT_OLD=1 VIZE_JSX_VDOM_CHILD_CAPTURE="$directory/base" cargo test --locked --profile ci -p vize_atelier_jsx --test vdom_child_values -- --nocapture) > "$directory/base.stdout" 2> "$directory/base.stderr" || status=1
if test "$status" = 0; then
  node "$VIZE_CI_FILES/original-vdom-child-values-and-parent-block-ownership-before-and-after.ts" "$directory"
fi
exit "$status"
