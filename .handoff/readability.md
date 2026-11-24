# Readability program handoff (2026-11-24, about 21:50 JST)

Delete `.handoff/` before this branch merges.

## State

| Item | Branch | State |
| --- | --- | --- |
| PR 1 (L0): ratchet, byte-custody registry, identity checkers | `test/readability-ratchet` | **#13638** open, CI running on `cf1e8c06e9`. Auto-merge is off; turn it on once required checks are green and the merged-tree check passes. |
| PR 2 (L1): house source style | `docs/house-source-style` | pushed, **no PR yet**. Open it after #13638's CI with `pr-bodies/pr2-house-style.md` (replace nothing: it already cites #13638). **Hold for maintainer review: no auto-merge.** |
| PR 3 (pilot): fresco, musea, doctor, marquette, curator, armature | `refactor/house-style-quiet-crates` (this branch) | pushed, **no PR yet**. Waits for #13638 to merge, plus the IR identity and release-size evidence. |

### Main-merge rule for #13638

Every merge to main can add issue-number comments and push #13638's merge group over its baseline (for example, #13616 added 11).
- Before queueing, and after each main merge, run `sweep/reseed-pr1.sh` from `rd-l0`. It merges `origin/main`, reseeds both TSVs and runs the tests. Then commit and push. Do not force-push.
- A watcher can repeat the merged-tree check: in a scratch worktree, `git merge origin/test/readability-ratchet`, then `node tools/commands/ci/readability-ratchet.ts --check`.
- The issue-number lines absorbed by the reseed are listed in `readability/absorbed-issue-lines.tsv`, mostly in `crates/vize/tests/lint_rule_option_extensions_cli.rs`, patina `config_rule_options` and `npm/builder/vite`. They are a follow-up cleanup PR; do not ask their authors.

## Decisions taken in PR 1

- Only `lines`, `issue-comments` and `issue-names` fail `check`. `long-functions`, `restating-comments`, `mixed-imports` and `hardcoded-limits` are advisory, because a replay showed they would have blocked 3 of 5 ordinary fixes (#13598, #13603, #13611). They flip to enforced in `enforcedCounters` (`tools/commands/ci/readability/counters.ts`) after PR 2 is adopted and the fleet brief tells agents.
- The issue-number check would have failed #13603, #13613 and #13611. Tell the fleet: no `(#1234)` in comments, file names or test names.
- Data lives in `tools/config/{readability-baseline,byte-custody}.tsv`, because `tools/ci` is not on main yet. It moves with `tools/commands/ci` later.

## Pilot (this branch): done and collected

Commit "wip: pilot sweep". Squash and retitle it `refactor(fresco, musea, doctor, marquette, curator, armature): apply the house style` when opening.

The changes:
- 20 issue-number comment lines removed (armature);
- `tests/sfc_attribute_boundary_11731.rs` renamed to `sfc_attribute_boundary.rs`;
- 99 restating comment lines deleted after review (the kept classes are listed in `pr-bodies/pr3-pilot.md`);
- 69 import blocks regrouped (`sweep/regroup_imports.py`, then `cargo fmt -p …`);
- 29 blank lines inserted between items (`sweep/item_spacing.py`).

Evidence so far (`readability/token-identity.txt`, `counts.txt`, `test-summary.txt`):
- Token identity on 105 files: 10 identical, 56 use-reordered, 38 doc-only (none on product-text items), 1 moved (the test rename), 0 different.
- Counters base → head: mixed-imports 69 → 0, restating-comments 141 → 42, issue-comments 20 → 0, issue-names 1 → 0. `lines` is unchanged; `parser/tests.rs` stays at 1,966 lines.
- `cargo test -p` (6 crates): 823 passed, 0 failed, 2 ignored, in 41 binaries. CI clippy (`--profile ci -D warnings -D clippy::wildcard_imports`) is clean. `cargo fmt --check` is clean. `source-file-lengths` reports no growth.

## Resume steps

1. **IR identity.** `sweep/pilot-evidence.sh ir` (it takes the fleet release lock and needs ≥ 60 GB free). The first attempt failed with `cargo reported no LLVM IR for vize_fresco`. Likely cause: `vize_fresco` builds a `cdylib` as well as an `rlib`, so `emitIr` in `tools/commands/levels/ir-identity.ts` does not find the `.ll` beside a hashed rlib. Investigate (try `--crate` per package, and check `target/release/deps/*.ll`). Fix it in the wrapper or with a per-crate `cargo rustc --crate-type rlib`, then rerun for all 6 crates. Expect `identical` with only Location line and column normalized.
2. **Release size.** After pushing, run `gh workflow run shipped-profile-measure.yml -f base_sha=<full main sha> -f head_sha=<full pilot sha> -f twins=false`. The `vize` CLI and `vize_vitrine` bytes must be identical.
3. **After #13638 merges.** Rebase this branch, delete `.handoff/`, run `node tools/commands/ci/readability-ratchet.ts --write` (it lowers the six areas), squash, fill the `<IR>`, `<SIZE>` and `<TESTS>` cells of `pr-bodies/pr3-pilot.md`, open the PR, and enable auto-merge only once the evidence is clean.
4. **Next PR (pilot part 2).** Move the 30 sibling `*_tests.rs` / `tests_*.rs` files and the 2 `mod.rs` files to `foo/tests.rs` / `foo.rs`. The `super::` paths change, `parser/tests.rs` is over the limit (don't let it grow), and the moves need test-list parity.
5. Keep at most one PR in CI. Check `/bin/df -g /Users` ≥ 60 before cargo. Never admin-merge.

## Files here

- `readability/{plan,style-guide,inventory}.md`, `readability/scripts/`: the original plan and inventory scripts.
- `readability/sweep/`: pilot codemods, `restating.ts` (lists ratchet candidates), `pilot-evidence.sh` (`static` / `build` / `ir`), `reseed-pr1.sh`.
- `readability/pr-bodies/`: drafted bodies for PR 2 and PR 3.
