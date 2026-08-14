# Mutation baseline — cargo-mutants pilot pair (P0-12)

The recorded baseline for TS-14 (mutation score, [test-suites.md](./test-suites.md)):
`cargo-mutants` run over the pilot pair `vize_carton` + `vize_relief`, per the
assurance doctrine's "tests are themselves tested" rule
([assurance.md](../assurance.md) §4). The measured scores are the initial
ratchet floors in [budgets.toml](./budgets.toml) `[mutation]` — they may only
rise; a surviving mutant in later work is a missing or lax test, not noise.

## Run provenance

- cargo-mutants 27.1.0 (`cargo install cargo-mutants --locked`), rustc 1.95.0
- Command per crate: `cargo mutants -p <crate> --jobs 2 --timeout-multiplier 3`
  (plus `-o <dir>` to relocate `mutants.out/`)
- Test scope: cargo-mutants default — the mutated package's own suite
  (`--test-workspace` unset), so the score measures each crate's _direct_
  tests, not incidental coverage from downstream crates
- Measured tree: davinci @ `baee106f3` (post-P0-8 merge). `vize_relief` is
  byte-identical between that tree and the phase-0 exit-gate head
  (`git diff baee106f3..baf55ea7d -- crates/vize_relief` is empty), so its
  score transfers as-is. `vize_carton` gained P0-9's `Span` type and P0-11's
  profiler attribution after the measurement (+2,002/−208 lines across 15
  files, tests included), so its seed predates that code — the
  reference-runner re-record below re-measures at its own head
- Date: 2026-08-14; machine: Apple M2 Max (12 cores, 96 GB), macOS 26.5 — a
  shared dev machine. The `vize_carton` run overlapped the phase-0 corpus
  re-baseline sweep (hence the conservative `--jobs 2`); the `vize_relief`
  run followed on the same machine after the sweep. Scores are
  parallelism-independent; wall times are machine-bound and not comparable
  across machines
- Wall clock: `vize_carton` ≈1h00m, `vize_relief` 1m10s. The carton
  invocation was torn down by its harness right after the final verdict, so
  its `outcomes.json` carries no end timestamp — every mutant has a verdict
  (306 + 300 + 48 + 27 = 681, the full population) and the wall clock is
  taken from run start to the last outcomes write. The timeout tally is
  load-sensitive: carton's 48 timeouts were recorded under the concurrent
  sweep, relief's 0 on the quiet machine — one more reason the enforced
  floors get re-recorded on the reference runner

## Scores

| crate         | total | caught | missed | timeout | unviable | viable |  score |
| ------------- | ----: | -----: | -----: | ------: | -------: | -----: | -----: |
| `vize_carton` |   681 |    306 |    300 |      48 |       27 |    654 | 0.4678 |
| `vize_relief` |    76 |     36 |      7 |       0 |       33 |     43 | 0.8372 |

**Definitions.** `viable` = `total` − `unviable` (mutants that fail to
compile prove nothing about the tests and are excluded). `score` =
`caught` / `viable`, truncated — never rounded up — to four decimal places
before it becomes a floor (exact fractions: 306/654 and 36/43). A `timeout`
counts against the score (a mutant that stalls the suite is not a caught
mutant) but is tallied separately so a timeout-driven dip is distinguishable
from a lost assertion.

## Missed mutants

Source-order listings (file, then line). The full per-mutant record —
including diffs and per-mutant logs — is reproducible with the commands
above; cargo-mutants writes it to `mutants.out/` (`missed.txt`,
`outcomes.json`).

### vize_carton — first 25 of 300 missed

| location                                | function                                   | mutation                                                                                                  |
| --------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| `vize_carton/src/allocator.rs:36`       | `Allocator::with_capacity`                 | `replace Allocator::with_capacity -> Self with Default::default()`                                        |
| `vize_carton/src/allocator.rs:52`       | `Allocator::as_bump`                       | `replace Allocator::as_bump -> &Bump with Box::leak(Box::new(Default::default()))`                        |
| `vize_carton/src/allocator.rs:61`       | `Allocator::reset`                         | `replace Allocator::reset with ()`                                                                        |
| `vize_carton/src/allocator.rs:76`       | `<impl Deref for Allocator>::deref`        | `replace <impl Deref for Allocator>::deref -> &Self::Target with Box::leak(Box::new(Default::default()))` |
| `vize_carton/src/allocator.rs:84`       | `<impl AsRef<Bump> for Allocator>::as_ref` | `replace <impl AsRef<Bump> for Allocator>::as_ref -> &Bump with Box::leak(Box::new(Default::default()))`  |
| `vize_carton/src/corsa_resolver.rs:105` | `resolve_corsa_executable`                 | `replace resolve_corsa_executable -> Result<PathBuf, CorsaResolveError> with Ok(Default::default())`      |
| `vize_carton/src/corsa_resolver.rs:114` | `discover_corsa_in_ancestors`              | `replace discover_corsa_in_ancestors -> Option<PathBuf> with None`                                        |
| `vize_carton/src/corsa_resolver.rs:114` | `discover_corsa_in_ancestors`              | `replace discover_corsa_in_ancestors -> Option<PathBuf> with Some(Default::default())`                    |
| `vize_carton/src/corsa_resolver.rs:134` | `normalize_corsa_path_with_discovery`      | `replace match guard resolved != path with true in normalize_corsa_path_with_discovery`                   |
| `vize_carton/src/corsa_resolver.rs:141` | `platform_suffix`                          | `replace platform_suffix -> &'static str with "xyzzy"`                                                    |
| `vize_carton/src/corsa_resolver.rs:226` | `discover_runtime`                         | `replace discover_runtime -> Option<PathBuf> with None`                                                   |
| `vize_carton/src/corsa_resolver.rs:226` | `discover_runtime`                         | `replace discover_runtime -> Option<PathBuf> with Some(Default::default())`                               |
| `vize_carton/src/corsa_resolver.rs:252` | `dev_paths_enabled`                        | `replace dev_paths_enabled -> bool with true`                                                             |
| `vize_carton/src/corsa_resolver.rs:252` | `dev_paths_enabled`                        | `replace dev_paths_enabled -> bool with false`                                                            |
| `vize_carton/src/corsa_resolver.rs:253` | `dev_paths_enabled`                        | `replace && with \|\| in dev_paths_enabled`                                                               |
| `vize_carton/src/corsa_resolver.rs:253` | `dev_paths_enabled`                        | `delete ! in dev_paths_enabled`                                                                           |
| `vize_carton/src/corsa_resolver.rs:253` | `dev_paths_enabled`                        | `replace != with == in dev_paths_enabled`                                                                 |
| `vize_carton/src/corsa_resolver.rs:261` | `compile_time_workspace_root`              | `replace compile_time_workspace_root -> Option<PathBuf> with None`                                        |
| `vize_carton/src/corsa_resolver.rs:261` | `compile_time_workspace_root`              | `replace compile_time_workspace_root -> Option<PathBuf> with Some(Default::default())`                    |
| `vize_carton/src/corsa_resolver.rs:433` | `scrape_pnpm_store`                        | `replace \|\| with && in scrape_pnpm_store`                                                               |
| `vize_carton/src/corsa_resolver.rs:472` | `find_in_home_locations`                   | `replace find_in_home_locations -> Option<PathBuf> with None`                                             |
| `vize_carton/src/corsa_resolver.rs:472` | `find_in_home_locations`                   | `replace find_in_home_locations -> Option<PathBuf> with Some(Default::default())`                         |
| `vize_carton/src/corsa_resolver.rs:503` | `find_in_npm_global_root`                  | `replace find_in_npm_global_root -> Option<PathBuf> with None`                                            |
| `vize_carton/src/corsa_resolver.rs:503` | `find_in_npm_global_root`                  | `replace find_in_npm_global_root -> Option<PathBuf> with Some(Default::default())`                        |
| `vize_carton/src/corsa_resolver.rs:507` | `find_in_npm_global_root`                  | `delete ! in find_in_npm_global_root`                                                                     |

### vize_relief — 7 missed

| location                                    | function                                 | mutation                                                   |
| ------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------- |
| `vize_relief/src/options.rs:51`             | `TemplateSyntaxMode::is_quirks`          | `replace TemplateSyntaxMode::is_quirks -> bool with true`  |
| `vize_relief/src/options.rs:51`             | `TemplateSyntaxMode::is_quirks`          | `replace TemplateSyntaxMode::is_quirks -> bool with false` |
| `vize_relief/src/relief/control_flow.rs:69` | `<impl Drop for IfBranchNode<'_>>::drop` | `replace <impl Drop for IfBranchNode<'_>>::drop with ()`   |
| `vize_relief/src/relief/control_flow.rs:69` | `<impl Drop for IfBranchNode<'_>>::drop` | `delete ! in <impl Drop for IfBranchNode<'_>>::drop`       |
| `vize_relief/src/relief/control_flow.rs:97` | `<impl Drop for ForNode<'_>>::drop`      | `delete ! in <impl Drop for ForNode<'_>>::drop`            |
| `vize_relief/src/relief/control_flow.rs:97` | `<impl Drop for ForNode<'_>>::drop`      | `replace <impl Drop for ForNode<'_>>::drop with ()`        |
| `vize_relief/src/relief/elements.rs:62`     | `<impl Drop for ElementNode<'_>>::drop`  | `replace <impl Drop for ElementNode<'_>>::drop with ()`    |

### Reading the misses

`vize_carton`'s 0.4678 is a _direct-test_ coverage number, and the misses
concentrate where the crate is a shared foundation exercised only from
downstream crates (which the default test scope deliberately excludes):

- **Classifier tables and predicates** — `dialect.rs` (83 missed),
  `i18n.rs` (74), `general.rs` (47) account for 204 of the 300. These are
  dialect profiles, locale/domain catalogs, and `phf`-set membership
  predicates; survivors like `is_petite_vue -> true` **and** `-> false`, or
  `is_reserved_prop`/`is_builtin_tag` surviving both polarities, prove no
  in-crate test pins either outcome.
- **Host-environment discovery** — `corsa_resolver.rs` (24): pnpm-store
  scraping, npm global roots, home-directory probing. Inherently
  host-dependent branches with no hermetic in-crate tests.
- **LSP client plumbing** — `lsp.rs` (24), plus `flags.rs` (19) and the
  long tail (`dom_tag_config.rs` 8, `allocator.rs` 5, `hash.rs` 5,
  `path.rs` 4, `line_index.rs` 3, three singletons). The `allocator.rs`
  five are the bump-allocator surface (`with_capacity`/`reset`/`Deref`) —
  used by every downstream crate, asserted by none of carton's own tests.

`vize_relief`'s 7 misses are two coherent gaps: five are `Drop` impls of
arena nodes (`IfBranchNode`, `ForNode`, `ElementNode`) — deleting the drop
body changes nothing any test observes — and two are
`TemplateSyntaxMode::is_quirks` surviving both polarities, i.e. no test
distinguishes quirks-mode from standard-mode behavior.

These themes are the first ratchet workload: each is a missing-test work
item, and the floors above only move when those tests land.

## Nightly-lane CI job — pending, lands with the reference-runner baselines

Phase-0's queue is hot; workflow additions ride along with the Blacksmith
reference-runner baseline re-record (P0-4) rather than landing solo. Until
then the job definition below **is** the deliverable: it is the reviewed
shape of `.github/workflows/mutants.yml`, and the PR that adds it re-records
the `[mutation]` floors on `blacksmith-32vcpu-ubuntu-2404` in the same
change (scores are parallelism-independent, but the reference runner is the
comparable substrate every other budget uses). Two notes bind the snippet:

- Scheduled workflows trigger from the default branch only, so this lands
  once the davinci tree (this directory, `budgets.toml` included) is on the
  branch the schedule runs from.
- cargo-mutants exits 2 when mutants survive; below a 1.0 score that is the
  expected outcome, so the job treats 2 as data and lets the ratchet
  comparison give the verdict.

```yaml
name: Mutants

on:
  workflow_dispatch:
  schedule:
    # Nightly, offset from the fuzz lane (23 4 * * *) so the two heavy
    # scheduled jobs do not contend for runners.
    - cron: "41 3 * * *"

concurrency:
  group: mutants-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

env:
  CARGO_INCREMENTAL: 0
  CARGO_TERM_COLOR: always

jobs:
  mutants:
    name: Mutation ratchet ${{ matrix.crate }}
    runs-on: blacksmith-32vcpu-ubuntu-2404
    timeout-minutes: 300
    strategy:
      fail-fast: false
      matrix:
        include:
          - crate: vize_carton
            floor-key: carton_score
          - crate: vize_relief
            floor-key: relief_score
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6

      - uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable
        with:
          toolchain: stable

      - uses: ./.github/actions/setup-rust-sticky-cache
        with:
          key: mutants-${{ matrix.crate }}
          cache-key-suffix: ${{ runner.os }}-${{ runner.arch }}

      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked --version "^27.1"

      - name: Run cargo-mutants (${{ matrix.crate }})
        run: |
          # Exit 2 (missed mutants) is a verdict for the ratchet step, not a
          # job failure; anything else non-zero is an infrastructure failure.
          # --jobs 4 on the 32-vcpu runner: the score is parallelism-
          # independent (the baseline was recorded at --jobs 2), parallelism
          # only moves wall time; --timeout-multiplier 3 matches the
          # recorded baseline.
          status=0
          cargo mutants -p ${{ matrix.crate }} --jobs 4 --timeout-multiplier 3 \
            -o mutants-run || status=$?
          if [ "$status" -ne 0 ] && [ "$status" -ne 2 ]; then
            echo "::error title=cargo-mutants failed::exit $status is not a mutation verdict"
            exit "$status"
          fi

      - name: Enforce the budgets.toml ratchet floor
        env:
          FLOOR_KEY: ${{ matrix.floor-key }}
          CRATE: ${{ matrix.crate }}
        run: |
          floor="$(awk -v key="$FLOOR_KEY" '
            /^\[/ { in_section = ($0 == "[mutation]") }
            in_section && $1 == key { print $3 }
          ' davinci-road/plan/budgets.toml)"
          if [ -z "$floor" ]; then
            echo "::error title=Missing floor::$FLOOR_KEY not found in budgets.toml [mutation]"
            exit 1
          fi
          score="$(jq -r '(.caught) / (.total_mutants - .unviable)' \
            mutants-run/mutants.out/outcomes.json)"
          echo "$CRATE mutation score: $score (floor: $floor)"
          if ! awk -v s="$score" -v f="$floor" 'BEGIN { exit !(s >= f) }'; then
            echo "::error title=Mutation ratchet breach::$CRATE score $score fell below the recorded floor $floor — a test lost its teeth; see the missed-mutants artifact"
            exit 1
          fi

      - name: Upload missed-mutant listing
        if: always()
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
        with:
          name: mutants-${{ matrix.crate }}
          path: |
            mutants-run/mutants.out/missed.txt
            mutants-run/mutants.out/timeout.txt
            mutants-run/mutants.out/unviable.txt
            mutants-run/mutants.out/outcomes.json
          if-no-files-found: ignore
```
