# Handoff: delete Glyph's unused `native_doc` route (#6847, slice F5)

Delete this file before merge.

## State (2026-11-19)

One commit on top of `origin/main` 8b09dea8b1 (`refactor(glyph): delete the unused native_doc route (#6847)`):
182 files, +10 / -41,902. The code is done. No cargo build or CI measurement has run yet. The disk stayed at
58 GB free, under the 60 GB build floor, and the CI queue had more than 40 runs waiting.

### Deleted

- `crates/vize_glyph/src/native_doc/` and `native_doc.rs` (5,885 lines), and `pub mod native_doc;` in `lib.rs`
- Every `crates/vize_glyph/tests/native_*` file and dir, `document_layout.rs`, `tests/support/`, and the fixtures
  `native-vue1-text-doc-1.0.28.json`, `native-vue2-scriptless-sfc-2.7.16.json`, `native-vue2-text-doc-2.7.16.json`
  (33,988 test lines)
- `[[test]] native_vue2_sfc_diagnostics_drop` in `crates/vize_glyph/Cargo.toml`. All other dev-deps stay, because
  benches and the remaining tests still use them: `davinci_harness` (bench), `sha2` (print-width tests), `insta`,
  `criterion`, `davinci_test_support`. Cargo.lock does not change.
- `.github/actions/test-native-{vue1-text-document,vue2-text-document,vue2-scriptless-sfc,historical-text-documents,historical-sfc-documents}`.
  The two wrappers only sequenced the three captures. Their single caller step is gone from
  `.github/actions/test-full-tooling/action.yml`.
- `tests/tooling/native-{vue1-text-document-capture-workflow,vue1-text-document-oracle,vue2-text-document-oracle,vue2-scriptless-sfc-oracle}.test.ts`
  and the helpers that became unused: `support/{vue1-text-document-judge,vue1-pinned-getter,vue2-whole-sfc-oracle,vue2-whole-sfc-observations}.ts`.
  `vue2-pinned-{oracle,runtime}.ts` stay, because other tests use them.
- README: the native_doc entry-point paragraph and the "opt-in arena-backed document IR" highlight

### Updated lists

- `historical-sfc-documents` lead step: removed from `TOOLING_LEAD_STEPS` (`tooling-test-shards.ts`, along with the
  now-dead `output === "always"` branch), from `CURRENT_LEAD_NAMES` (`update-tooling-test-timings.ts`) and from
  `tests/tooling/fixtures/tooling-test-timings.json`, together with the 8 timing rows of the deleted tests.
  `tooling-test-shards.test.mjs` and `tooling-test-timings.test.ts` were adjusted to match.
- `github-workflows-full-tooling-gate.test.mjs`: dropped the deleted actions; counts went 4→3 independent steps and
  14→13 execution steps.
- `tests/_fixtures/source-layout/10418.json`: dropped the 9 rows whose live owners were deleted. The test requires
  every live owner to exist.
- `docs/davinci/plan/v-on-corpus/crates--vize_glyph.tsv`: regenerated with `node tools/support/compat/davinci/v-on-corpus.ts --write`.
- Left alone on purpose: `docs/davinci/decisions/*` and the `formatter/sfc/qualification-20261105` manifest case.
  That case's provenance pins sha 8c83f43. Its witness path now dangles, but three other witnesses already dangle
  and no validator checks that witness paths exist.

### Gates already passed locally

- `cargo fmt --all -- --check`: ok
- `SOURCE_LENGTH_BASE_REF=8b09dea8b1 node tools/commands/ci/source-file-lengths.ts`: ok
- `rust-script tools/commands/davinci/assertion-lint.rs`: OK. There were no native_doc ratchet or allowlist entries.
- Tooling: `github-workflows-*`, `tooling-*`, `source-layout-relocations`, `davinci-storage-policy`, `davinci-v-on-*`,
  `davinci-matrices` and the other readers of the changed files: 304 pass. The 1 failure is
  `lsp-code-action-kinds` (create-vue submodule not hydrated locally; unrelated).

## Next steps

1. Rebase onto `origin/main` and re-grep for stragglers:
   `git grep -n -e native_doc -e native-vue1-text -e native-vue2-text -e native-vue2-scriptless -e historical-sfc-documents -- ':!docs/davinci/decisions'`.
2. Disk must be at least 60 GB free. Then run
   `cargo clippy -p vize_glyph --profile ci -- -D warnings -D clippy::wildcard_imports` and `cargo test -p vize_glyph`.
   Watch for dead-code warnings in template helpers that only native_doc used.
3. Build `cargo build --locked -p vize --profile ci` for base 8b09dea8b1 (or the new merge base) and for head, using
   one target dir. Copy both binaries, then run
   `scratchpad/formatter/parity.sh <repo> <base-bin> <head-bin> <outdir>`, then
   `node tests/differential/build-receipt.ts` and `node --test tests/tooling/differential-formatter-cli.test.mjs`.
4. Release size: hold the fleet lock `/private/tmp/claude-501/vize-release-build.lock`. Build
   `cargo build --locked --release -p vize` for base and head and record bytes and sha256. Delete `target/release`,
   then `rmdir` the lock. Adapt `scratchpad/formatter/release-size.sh`, which hardcodes another worktree and branch.
5. Instruction rows: the release profile is binding. Use `gh workflow run shipped-profile-measure.yml -f base_sha=<40hex> -f head_sha=<40hex>`
   once `git ls-tree origin/main .github/workflows/shipped-profile-measure.yml` shows it. It was still missing on main
   at handoff. The ci-opt profile is informational: dispatch `level-instruction-counts.yml` on a `measure/...` base
   pointer branch (`git commit-tree <base>^{tree} -p <base>`) and on head, interleaved base, head, base twin, head twin.
   Wait while more than 40 runs are queued.
6. Open the PR with title `refactor(glyph): delete the unused native_doc route (#6847)`. No `!`: the crate is
   `publish = false` and no workspace product calls the module. Reference
   https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-7007413822 and follow the #13340/#13407 table format.
   Add the PR performance observation wall/CPU/RSS/ci-opt size tables. Enable auto-merge only once every metric is
   clean, and leave it off while the release-profile measurement is pending.
