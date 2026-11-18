You are a worker for the Davinci completion leader on ubugeeei-prod/vize. Do NOT spawn agents. First read and follow the common lane brief (environment, sandbox, gh/git unsandboxed, cargo-slot wrapper, toolchain, GitHub and zero-regression rules):
/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad/lane-brief.md

## Mission: the linter on the shared levels (#6848), rule families onto the L2 markup facade
Maintainer priority (2026-11-10, #6826): SFC, JSX and TSRX lower to the same L1/L2, and the linter runs one implementation over them with no per-input branches. Gate: `products.linter` (#6848, "Patina runs syntax rules on L1 and semantic rules on L2").

Current state (generate it yourself with `node tools/support/compat/davinci/rule-parity.ts --write --out-dir $TMPDIR/rule-parity`):
371 rules; template-family 263. On JSX: `ir` 65 + `ir-lowered` 18 = 83 rules implement `MarkupRule` (`crates/vize_patina/src/markup.rs`, the zero-copy facade with L2, Relief and OXC backends). `fallback` = 95 rules have only template-visitor hooks, so JSX runs them over a JSX root lowered to Relief (`legacy_keep_mask` in `crates/vize_patina/src/linter/engine.rs`). TSRX lint (`linter/engine/tsrx.rs`) runs only markup rules over L2, so fallback rules never run on `.tsrx` at all.

Work, one rule family per PR (for example a11y, then html, then vue structural):
1. Port `fallback` rules to `MarkupRule`, keeping the template-visitor path for SFC only where the SFC lane still needs it. Prefer rules whose checks are input-neutral (`neutral-core-candidate` in the report).
2. Output must be byte-identical on every existing input: SFC, JSX/TSX and the fix-history oracles (`crates/vize_patina/tests/lint_history.rs` plus the linter fix-history packs under `crates/vize_patina/tests/fixtures/`). Use the facade's `differential` lane (`crates/vize_patina/src/markup/differential*`) to prove Relief and L2 answer identically for each ported rule. A rule that now also reports on `.tsrx` is an intended change; add a TSRX test for it and say so in the PR.
3. Regenerate the rule-parity report and quote the before and after lane counts in each PR body (`fallback` down, `ir`/`ir-lowered` up).
4. Perf: lint must not get slower. `pr-performance-observation.yml` runs a perf-linter quiet-runner bench on each PR, so read its result. For engine changes, also dispatch `benchmark.yml` (read its inputs; it takes base_sha/head_sha with --ref at the head branch) and quote the run id.

## Notes
- Open leader-owned PRs touch nearby Patina files. Keep your diffs away from these regions and rebase often:
  - #13304, #13308 and #13313 touch `crates/vize_patina/src/markup/l2.rs` (the `L2Template` constructors, which will move to `markup/l2/construct.rs`), `linter/engine.rs`, `linter/engine/parse_diagnostics/**` and the native test suites.
  - #13306 touches the duplicate-attribute severity, and #13309 touches `parse_diagnostics/compatibility/**`.
  - The TSRX lane may add L2 ops with match arms in `markup/**`.
- CI: at most one workflow_dispatch in flight. If the orchestrator or the lead says runners are starved, hold pushes and reruns.
- Disk is tight (about 23 GB free). Keep your target dir under about 12 GB, and clean stale test binaries.
- Local Mac baseline: `vize_patina --test integration lint_history::full_current_results_and_independent_fix_applications` fails on origin/main too (row `multiline-v-for-source-9880`, type-aware environment). Diff your failure set against origin/main's.
- Create your own worktree from origin/main (`git -C /Users/ubugeeei/Source/github.com/ubugeeei-prod/vize worktree add ../vize.lint-l2 -b lint/markup-<family> origin/main`, unsandboxed). Never build in the main checkout. At most 2 open PRs at a time. Arm with `gh pr merge <N> --auto` and follow each until MERGED.
- Do not touch these leader-owned PRs: #13231, #13262, #13303-#13309 and #13313.
- Record decisions as short English comments on #6848.

## Report back (final message, concise English)
PRs with state and merge SHAs, rules ported per PR, rule-parity lane counts before and after, evidence (tests, perf run ids), and anything that needs a maintainer decision (as options).
