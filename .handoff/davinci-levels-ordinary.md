# Handoff: davinci-levels-ordinary (#6844 S7, the paired normal-script bindings switch)

The earlier PR #13253 was closed by the coordinator (no drafts). Open a fresh,
ready PR from this branch only when it meets the full merge bar.

## State

Commits on top of main, `2de08e4787` at the time of writing. The branch needs a
rebase first.

1. `refactor(sfc)`: `compile/bindings/ordinary.rs` makes L2 `OrdinaryFacts` the
   source of a paired normal script's bindings. `collect_normal_script_bindings`
   is removed. Also in this commit:
   - L2: a named `default` import from `.vue` is now SetupConst.
   - The reviewed source pin and hash are updated.
2. `docs(davinci)`: the decision page notes the #5067 change.
3. `perf(sfc)`: `OrdinaryFacts::of_module` reads only the top-level statements
   instead of running the full File walk. L2's corpus test
   `statement_only_read_equals_the_walk_on_every_corpus_normal_script` finds it
   equal to the walk on all 101 corpus normal scripts.
4. `test(sfc)`: pinned DOM modules
   `tests/_fixtures/differential/compiler/{normal-script-ambient-declare-6844,named-default-vue-import-6844}`.
   They match `@vue/compiler-sfc` 3.6. `VIZE_ORDINARY_FIXTURE_WRITE=1` rewrites
   them.

Local verification passed for all of the following:
- `cargo test -p vize_atelier_sfc --no-fail-fast`: 1,268 pass.
- The L2 ordinary, setup facts, bindings and contract tests.
- clippy, with CI flags, on the touched files.
- fmt.
- The tooling tests storage-policy, source-file-lengths, docs-markdown-links,
  sfc-parse-attributes-reference/bootstrap and level-input-placement.

## Performance so far

All numbers are local ci-opt instructions retired on rustc 1.100. Local wall
times are not evidence.

- The first version, with the full L2 walk, cost +1.2% (29.38e9 → 29.72e9). The
  CI 1000-SFC observation flagged it as Compile SFC 1.070.
- After `of_module`, the paired corpus files without the four
  deliberately-changed ones (58k compiles, 7 runs): base 20.849e9, head
  20.827e9 (−0.1%).
- All 32 paired files (64k compiles): +0.18%. That difference comes only from
  the four files whose output changes on purpose (the `_resolveComponent` work
  Vue does too).
- Micro-bench (64k analyses of the 32 paired normal scripts):
  - bare OXC parse: 4.05–4.18e9
  - L1 parse: 4.04–4.09e9
  - new route: 4.16–4.18e9
  - old full walk: 4.55e9

## Next steps (coordinator decision, 2026-11-17)

1. Add a **default-off flag**.
   - Restore main's `collect_normal_script_bindings` with its helpers verbatim
     in `compile/bindings.rs` (it was 398 lines on main), so flag-off is
     byte-identical to main.
   - Make `bindings::ordinary()` dispatch on the flag: a `OnceLock` read of
     `VIZE_DAVINCI_ORDINARY_BINDINGS` (`1` = on, `0` = off), with a const
     default.
   - Keep `compile.rs` as it is on this branch, so the source pin does not
     move again.
   - For the ambient fixture test, use a `cfg(test)` thread-local override
     instead of setting environment variables.
   - The named-default fixture comes from the shared L2 classifier (setup
     route), so it does not depend on the flag.
2. Once the CI pause lifts:
   - Push and open a ready PR.
   - Let `pr-performance-observation` run.
   - Dispatch `level-instruction-counts.yml` once.
   - Run the **full real-project matrix** and the **compiler fix-history
     corpus** with the flag ON. Do that on a branch commit whose const default
     is on, because the matrix workflow takes no env input.
3. If the matrix, fix-history, the 1000-SFC observation and the instruction
   counts are all green with no regression, flip the default to on in this
   same PR. Otherwise ship flag-off and file issues for the gaps.
4. After it merges, post a progress comment on #6844 and send the coordinator a
   one-line update.

## PR body draft

The body is in `pr-body` below. Replace the `PENDING` line with the CI numbers.

### pr-body

S7 of the #6844 migration order, second half. When a normal `<script>` sits
beside `<script setup>`, the template also sees the normal script's runtime
bindings. The SFC compiler now reads them from the native L2 `OrdinaryFacts`
(#12699), and `collect_normal_script_bindings` is gone. That function was the
compiler's own second OXC walk, with a name-based `ref`/`reactive` guess. (With
the flag, it remains as the default-off fallback until the matrix is green.)

- `compile/bindings/ordinary.rs` parses the block once with L1 under its
  `lang`.
- L2's `OrdinaryFacts::of_module` reads the facts from the top-level statements
  that the File walk would hand `OrdinaryObserver`. A corpus test proves the
  two equal on 101 normal scripts.
- The first declaration of a name wins. A script the levels refuse exposes
  nothing.
- Vue's `compileScript` rules apply:
  - `ref`, `computed` and `reactive` count only when imported from `vue`;
  - `let` is `setup-let`;
  - ambient `declare` has no runtime binding;
  - a named `default` import from `.vue` is `setup-const` (L2 shared
    classifier).

Behavior changes, both pinned and both matching `@vue/compiler-sfc` 3.6:
- `vue-language-tools` `tsc/#2206`, `#4820` and `events` declare their
  components ambiently. Their components now resolve through
  `_resolveComponent`, where the old code returned names that do not exist at
  runtime.
- `tsc/#5067` reads its named default import directly.

Evidence:
- Corpus dump: 5,988 compiles, of which 5,976 are byte-identical; the other 12
  are the four files above, in DOM, SSR and Vapor.
- Tests:
  - `cargo test -p vize_atelier_sfc`: 1,268 pass.
  - L2: ordinary, setup facts, bindings and contract tests pass. The
    setup-bindings oracle inventory grows from 1,949 to 1,951 (the two named
    default `.vue` imports).
- Performance: the numbers above.
- CI 1000-SFC observation and instruction counts: PENDING.

Refs #6844

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Measurement recipe (the scratchpad does not carry over)

### Local corpus bench

- Write an uncommitted test, `crates/vize_atelier_sfc/tests/zz_local_corpus_dump.rs`:
  - collect every `.vue` under `tests/_fixtures`, `tests/fixtures`, `crates`,
    `playground`, `examples` and `npm`, skipping `node_modules`, `target`,
    `dist` and `.git`;
  - filter with `DUMP_FILTER=both` (script and script setup);
  - `DUMP_EXCLUDE` takes comma-separated path substrings to skip;
  - `BENCH_ROUNDS=N` compiles each file N times with default options;
  - `DUMP_OUT=dir` dumps code, bindings and errors per DOM/SSR/Vapor shape.
- Build it with `cargo test --profile ci-opt --no-run`.
- Get the base binary by checking out origin/main's `crates/vize_atelier_sfc/src`
  and `davinci/vize_l2/src` over the branch.
- Run the two binaries alternately under `/usr/bin/time -l`, reading
  "instructions retired".

### CI instruction-count comparison

- Download the `instruction-counts-*` and `sfc-instructions-*` artifacts of two
  `level-instruction-counts.yml` runs (or of merge-group Check runs).
- Compare the per-row medians of `measurement.json` `runs[*][name].instructions`.

### Move-only proof under ci-opt drift

ci-opt uses 16 codegen units, so module moves shift counts. Measure both sides
again with `codegen-units = 1`, using a temporary branch that edits
`[profile.ci-opt]`. Delete the temporary branch afterwards.
