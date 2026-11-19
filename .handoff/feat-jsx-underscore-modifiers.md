# Handoff: feat/jsx-underscore-modifiers (J3b of #6885, fixes #12313)

Delete before merge. One commit on `origin/main` (`fix(jsx): split _ modifiers ...`), no PR yet.

## State
- Code is complete: the shared split is in `vize_l1_to_l2::framework::vue::jsx::modifiers` and is used by both the native lowering and the Relief lowering (`lower/attr.rs`).
- Pinned compiler differential case: `compiler/jsx/underscore-modifiers-12313`.
- Ledger rows W-JSX-1/2 are in `docs/davinci/plan/compile-waivers.md`.
- The vue-jsx-vapor oracle rows `events/*-modifier` and `directives/custom-modifiers` are now equivalent.
- Tests pass: vize_atelier_jsx 475, the corpus runner, vize_l1_to_l2 jsx, patina jsx+markup 321, maestro jsx 59, and canon jsx 54. Canon's fix-history jsx test needs Corsa, which was not installed locally.
- Clippy and the assertion lint are clean. rustfmt has been applied.
- Release-profile instructions (fat LTO): base run 44508605066 (bench head `10acc5f3f1`, fat), head run 44510414393 (`measure/jsx-fat-j3b`). The 101 existing rows are identical. The three JSX rows rise: vdom +2.88%, vapor +1.04%, babel +0.13%. A callgrind function diff shows the rise is v-on modifier codegen, handler parsing and `split_event_modifiers`, i.e. the fix's own work.

## Blocker (decision needed)
#13382 pins the JSX ceilings in `instruction-budgets.toml`, and the ratchet (`ratchetBudgets`) forbids raising a ceiling. The merge queue enforces ci-opt ceilings, so J3b cannot land after #13382 without a mechanism for authorized fix raises. For example, a raise table keyed to a waiver id that the methodology checker accepts. Ask the maintainer or coordinator which mechanism to use.

## Next
1. After the mechanism exists: rebase on main, raise the three JSX ceilings to J3b's measured ci-opt counts with the waiver reference, then run four interleaved ci-opt and release runs. Then open the PR using the body draft below and enable auto-merge.
2. PR body essentials: the change, the ledger, the fixture, the oracle rows, product suites, and a release-profile table with the callgrind function diff.
