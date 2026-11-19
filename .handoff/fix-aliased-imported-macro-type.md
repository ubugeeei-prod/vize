# Handoff: fix/aliased-imported-macro-type (#13386)

Paused 2026-11-19 (fleet stop for the machine switch). The work is unverified locally: no build ran, because disk was under the 60 GB rule.

## State

The single WIP commit (`wip(sfc): renamed type imports`, to be squashed and retitled `fix(sfc): ...`) contains:

- **The fix**, in `crates/vize_atelier_sfc/src/script/context/external_types.rs`.
  - `FileTypeSummary` gains `renamed: Vec<(local, imported)>`, filled for `import { Imported as Local }` specifiers.
  - After the root walk, `try_collect_imported_types_from_path` aliases each renamed local (`type_aliases[local] = imported`) when the local name is not already declared and the imported name was collected.
- **Unit test** `renamed_type_imports_resolve_through_the_imported_name`, in `external_types/tests.rs`.
- **Compiler differential fixture** `compiler/project/aliased-imported-macro-type-13386`, for the fix-title gate.
  - Files: `tests/_fixtures/differential/compiler/aliased-imported-macro-type-13386/`, its manifest entry, and a new cohort in `tests/differential/compiler-project.ts`.
  - `module.expected.txt` is HAND-WRITTEN from the 13110 shape and NOT yet verified against a source-built CLI.

## Next steps

1. Wait until `/bin/df -g /Users` shows ≥ 60 GB, then run `cargo test -p vize_atelier_sfc --lib renamed_type_imports`.
2. Run `cargo build --locked -p vize --profile ci` and `node tests/differential/build-receipt.ts`.
3. Run `node --test tests/tooling/differential-compiler-project.test.ts` and the compiler pack. Regenerate `module.expected.txt` from the real CLI output if it differs, and update its sha256 in the manifest.
4. Run the full `vize_atelier_sfc` tests, the `compile_history` runner and the compiler differential tests. All must be byte-identical apart from the new case.
5. Run CI clippy for `vize_atelier_sfc` and the assertion lint, and check source lengths. `external_types.rs` is 490 lines, close to the 500 limit.
6. Squash into `fix(sfc): resolve renamed type imports for macro types (#13386)`. Add no co-author: the reporter is the fleet agent.
7. Collect measurements:
   - instruction counts, release profile per the coordinator's rule (`atelier_*_compile_*` rows may move);
   - release size, via the `bench/levels-release-size` dispatch with `base_sha`/`head_sha`;
   - the PR performance observation.
8. Open the PR and enable auto-merge only with clean evidence.

## Open decision

- The fix adds code, so release size may grow by a few bytes. The coordinator rule says release size must not grow. A ruling is needed if it does.
