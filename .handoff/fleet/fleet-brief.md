# Vize Davinci fleet — common brief (2026-11-19)

The maintainer's order (verbatim): "davinci を本気で完成させてくれ．ただし動作もパフォーマンスも絶対にデグレするな"
and "PR には必ず計測結果を載せてくれ．auto merge".

Davinci is complete when every gate in `docs/davinci/completion-gates.md` passes on main.
Check with `node tools/commands/davinci/completion-gates.ts --issues`. Read `docs/davinci/charter.md`
and the decision records under `docs/davinci/decisions/` relevant to your track before designing anything.
Goal: the LLVM/MLIR of frontend language processing. Input-specific code ends at L2; every product
reads the shared levels and never parses on its own; speed is gated; layouts must be clean/"beautiful".

## Hard rules

1. **No behavior regression.** Outputs must stay byte- and diagnostic-identical on the differential
   corpus, the fix-history corpus and the real-project matrix. A deliberate change needs a pinned
   fixture plus a ledger row. A route switch stays behind a default-off flag until the full
   real-project matrix and fix-history corpus pass.
2. **No performance regression on ANY metric** — wall, CPU, RSS, instruction counts (every stage row of
   `level-instruction-counts.yml`), binary size. Tiny codegen-shift regressions (+0.003% size, +0.01%
   instructions) are NOT accepted: find the cause and remove it (e.g. constant-length copies, avoiding
   inlining-sensitive helpers, `#[inline]`/`#[cold]` placement), or restructure. Local wall times on
   this shared machine do not count as evidence; use CI runners.
   **Binary size rule (maintainer, 2026-11-19):** size is judged on the SHIPPED release profile
   (fat LTO, 1 codegen unit): it must not grow. Report ci-opt size too, but a ci-opt-only shift with a
   byte-identical (or smaller) release binary is not a regression — thin-LTO codegen-unit partitioning
   reshuffles on crate-hash changes. Measure release size with a probe build of base and head.
3. **Every PR body carries measurement results** (tables, not claims): at minimum
   - `level-instruction-counts.yml` dispatched on base and head (interleaved: base, head, base twin,
     head twin) with an A/A control; a table of every row that moved, and "other N rows identical".
   - The `PR performance observation` run (runs automatically on PRs; download its artifact /
     summary.md) for wall/CPU/RSS/size, with base and head interleaved.
   - Byte/diagnostic parity evidence for output-affecting code.
   - Test-/docs-/tooling-only PRs still state which product binaries are unchanged (e.g. identical
     `vize` binary hash base vs head) instead of tables.
   Follow the format of PR #13340 (body) and #13359.
4. **Auto-merge**: once the evidence shows no regression on every metric and required checks are green,
   enable auto-merge (`gh pr merge <n> --auto --squash`). Never admin-merge. No draft PRs, ever; open a PR
   only when ready. Close a not-ready PR and keep the branch.
5. **CI capacity is shared.** Push rarely (batch commits), at most ~2 dispatches in flight per agent,
   cancel your own superseded runs. If the queue backs up (`gh run list --status queued | wc -l` > 40),
   wait. Third-party (external reporter) fixes always have priority over this work.
6. **Main moves under you.** Other agents merge to main directly. `git fetch` before every push, rebase
   onto origin/main, never force-push a branch someone else pushes to. Run the CI gates PR runs skip
   (below) before pushing.
7. **`unsafe`** only with a `// SAFETY:` comment on every block and a differential fuzz test.
8. Codenames (`levels.codenames` gate) are being removed by a separate scripted rename
   (`refactor/remove-codenames`). Do not add NEW codename uses (Davinci, Folio, Ricalco, Disegno, Impeto,
   Sinopia, Spolvero, Stesura) in paths or lines outside `docs/davinci/`; do not start the rename yourself.
9. Agent-written Vue SFCs use type-based `defineProps<{...}>()`.
10. Write PRs/commits in English. End commit messages with
    `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and PR bodies with
    `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.

## Toolchain and local gates

- Rust: always `nix develop --command bash -c 'cargo ...'` from the repo/worktree root (pinned 1.100.0).
  Plain `cargo` fails. Use your worktree's own target dir; never share `CARGO_TARGET_DIR`.
- Disk is shared: prune `target/*/incremental` after pushes; keep free space > 30 GB (`/bin/df -h .`).
  **Disk protocol (2026-11-19, after the disk hit 4 GB free):** before any cargo build, check
  `/bin/df -g /Users` free ≥ 60 GB, else wait (Monitor until-loop) and prune your own caches.
  Release (fat-LTO) size probes: prefer a CI dispatch over a local build. A local release build only while
  holding the fleet lock: `mkdir /private/tmp/claude-501/vize-release-build.lock` (succeeds = you hold it; `rmdir`
  when done; if it is older than 90 min, it is stale). Delete probe targets and copied binaries as soon as
  the numbers are recorded; never keep more than one extra target dir per agent.
- Before pushing, run what PR CI skips:
  - CI clippy: `cargo clippy -p <crates> --profile ci -- -D warnings -D clippy::wildcard_imports`
    (no `--all-targets`; std `String`/`format!` denied — use `vize_l0::String`, `cstr!`; no slice/str indexing).
  - Assertion lint: `rust-script tools/commands/davinci/assertion-lint.rs` (rust-script 0.34.0; install with
    `cargo install rust-script --version 0.34.0 --root <dir>` inside nix develop).
  - Source lengths: `SOURCE_LENGTH_BASE_REF=$(git merge-base HEAD origin/main) node tools/commands/ci/source-file-lengths.ts`
    (files may not cross 500 lines; over-limit files may never grow).
  - Tooling tests that run the CLI: `cargo build --locked -p vize --profile ci` then
    `node tests/differential/build-receipt.ts` (redo after every commit/rebuild).
  - `fix(...)` PR titles need a pinned differential fixture per touched legacy product
    (`vize_canon`→typechecker, `vize_maestro`/`vize_resident`→lsp, `vize_patina`→linter, `vize_glyph`→formatter,
    `vize_atelier_*`/`vize_armature`/`vize_relief`→compiler). Manifest cases stay sorted by `pack ?? id`.
  - New tooling files are `.ts`, never `.mjs`.
  - `crates/vize_canon/src/virtual_ts/generator.rs` is byte-custody pinned; avoid editing it.
  - The merge queue runs the full Rust suite (~6,150 tests); run the full suites of crates you touch.
- Instruction ceilings run only in merge_group; measure with `gh workflow run level-instruction-counts.yml --ref <branch>`.
- A push to a branch whose PR is in the merge queue fails (GH006); dequeue first via GraphQL `dequeuePullRequest`.

## Reporting

Finish with a short report: PRs opened/merged (numbers), measurement summary per PR, what is left on your
track with concrete next steps, and any decision that genuinely needs the maintainer. If you stop with
WIP, push it to a branch with a `.handoff/<branch>.md` (deleted before merge) describing next steps.

## Later maintainer rules (2026-11-19)
- A listed bug fix (pinned fixture + ledger row) may raise ONLY the instruction rows whose fixture exercises the fixed construct, with a callgrind function diff showing the rise is the fix's own work. All other rows ≤ base; wall/CPU/RSS unchanged; release size must not grow.
- No metric trade-offs otherwise: "−0.1% instructions for +128 B release size" is a regression and is rejected.
- Native routes keep today's output bytes unless the change is a listed fix (e.g. JSX `c && <A/>` keeps today's keyed-fragment/comment output).
- **Instruction counts are judged on the RELEASE profile (fat LTO, 1 codegen unit), strictly: every row ≤ base.** ci-opt rows are reported for information only. (Same principle as size.)
- **Move/rename-only PRs** (no code change): pass if optimized LLVM IR is identical to base after normalizing symbol names/paths (use the committed IR-identity checker once it exists), output is byte-identical, and release size does not grow. Row shifts are reported for information. Any PR that also changes code stays strict.
- **Source maps**: every backend breaks lines on JS line terminators (LF, CR, CRLF, U+2028, U+2029). Changing SSR/Vapor maps to that is a listed output change (pinned fixtures + ledger rows).
- **Linter**: syntax rules run on a pure L1 engine (no L2 lowering needed); promote and complete `native/`. The L2 facade is for semantic rules.
