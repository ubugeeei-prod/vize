# Standalone test runtime inventory

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

This source-bound audit covers every committed `Cargo.toml` at the three
immutable heads below. The existing locked `@iarna/toml` 2.2.5 parser read
each explicit `[[test]]` and `[[bench]]` registration. It did not build or
execute Cargo targets. Every listed standalone test belongs to the root
workspace and has no `required-features`. The 26 `harness = false` benches
at each head are separate benchmark targets and receive no test-case credit.

| Source snapshot                                               | Manifests read | Standalone tests | Standalone benches |
| ------------------------------------------------------------- | -------------: | ---------------: | -----------------: |
| Main `dd6beada6373fc58af149e7e19cf0204710ec022`               |             53 |                2 |                 26 |
| Embedding provider `5df4a6c35a7f07e36c41e8fd5e20366a2bb691f5` |             54 |                3 |                 26 |
| File provider `7c4a919cbe91c98000afef619ad5dbef520a4694`      |             53 |                2 |                 26 |

The unique nonbench registrations and their source-reviewed status are:

| Package / Cargo target                           | Source                                                       | Discovery and named execution                                                                                                                                                                                                                                                                                                                   | Remaining evidence / owner                                                                                                                                                                                                                                                                                     |
| ------------------------------------------------ | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vize_patina` / `davinci_authored_markup_budget` | `crates/vize_patina/tests/davinci_authored_markup_budget.rs` | Main `dd6` advertises no case and runs measurements during listing. Independently reviewed correction `f81d1c3480387d1ffe33c3abf2216bff7cadbcac` advertises `authored_projection_reuses_ordinary_storage_and_holds_the_facade_budget`; ignored discovery is empty and exact execution invokes the unchanged body.                               | Sixteen instrumented protocol invocations passed. Exact `f81d` also executed the real allocation body successfully under hosted direct Cargo; fresh nextest inventory and assigned worker/JUnit remain pending. Patina allocation-harness owner.                                                               |
| `vize_l0` / `remark_zero_cost`                   | `davinci/vize_l0/tests/remark_zero_cost.rs`                  | Main `dd6` has an unconditional main and advertises no case. Independently reviewed correction `ab32aaca28411e64c7451b4ff4a8f7c2cd0adca4` advertises `remarks_preserve_zero_cost_and_the_positive_control`, with normal/ignored discovery and exact selection preserving the complete original 0/0/3 allocation law and three analysis remarks. | Sixteen instrumented protocol invocations and eight scoped real-body selections passed; the latter use 23 exact `dd6` modules with retained foundation type/support identities and do not establish fresh whole-Cargo compilation. Fresh nextest inventory and assigned worker/JUnit remain pending. L0 owner. |
| `vize_l1` / `embedding_observation_drop`         | `davinci/vize_l1/tests/embedding_observation_drop.rs`        | Provider `5df` runs its measurement main unconditionally and advertises no case. Independently reviewed correction `d4eab73a3500cfa177068ca8386d9d6cb2b08791` advertises `original_embedding_diagnostics_drop_before_their_arena` with the complete original Drop law unchanged.                                                                | Eighteen real process controls passed, including failing-law controls proving discovery does not execute the body. The L1 owner must fold this repair into its shared provider; fresh nextest inventory and assigned worker/JUnit remain pending.                                                              |
| `vize_l2` / `file_program_unwind_alloc`          | `davinci/vize_l2/tests/file_program_unwind_alloc.rs`         | Provider `7c4a` advertises `first_unit_interruption_adds_no_allocation`. Its reviewed normal/ignored listing and exact selection preserve the private baseline/guarded fresh-process modes.                                                                                                                                                     | Eleven real binary protocol controls passed. This is scoped provider evidence; fresh source-bound nextest worker execution remains pending. File owner.                                                                                                                                                        |

All four targets have real Cargo registrations. The missing protocol is a
missing nextest-discovered case, not an unregistered Cargo executable.
Static inventory counts and historical direct Cargo observations do not
establish nextest runtime case coverage. No allocation budget, counter scope,
parse warm-up, production route or native feature changes in this audit.

The exact `f81d` [hosted Rust job](https://github.com/ubugeeei-prod/vize/actions/runs/36956693434/job/110681112977) ran `cargo test --workspace` and the silent Patina allocation executable returned successfully before the next target. The Rust job succeeded; the full workflow failed its then-existing node-forge security audit. This establishes a real direct-Cargo allocation observation on `f81d`, with no nextest discovery or named-worker credit. The combined Patina/L0 repair must obtain its own source-bound receipts after replay onto the actual merged security main.

## Required hosted receipts

The [nextest custom-harness protocol](https://nexte.st/docs/design/custom-test-harnesses/)
requires normal terse discovery to print one `CASE: test` line, ignored
discovery to print nothing, and `CASE --nocapture --exact` to execute the
actual case. Discovery must not enter a measured window.

An ordinary PR uses `.github/workflows/pr-rust-checks.yml`:

1. `pr-rust-build`, **Build and archive affected Rust tests once**, includes
   the affected owning package in the source-bound archive.
2. Each **Rust tests (N/4)** worker verifies that archive's identity and runs
   nextest 0.9.146 with profile `pr` and partition `hash:N/4`. The Patina case
   is absent from the profile's exact exclusions. The actual partition
   number must come from hosted output, not a guessed hash.
3. Require the assigned worker's complete `PASS` line for the exact package,
   binary and case, plus its JUnit testcase. The artifact is
   `rust-test-shard-N-RUN_ID-RUN_ATTEMPT`; `junit.xml` is uploaded from
   `target/nextest/pr/`. An absent testcase or merely green empty partition
   gives no runtime credit.

A merge-group source build additionally lists the complete archive into
`rust-test-timings/workspace-tests.json`, uploaded with its source/timing
receipt. Its four workers use profile `full`, which has no default filter.
Require the same named case and worker JUnit on the actual queue source.

Manual or scheduled full Check takes a different route: `clippy-and-test`,
**Test**, invokes `test-rust-workspace-differential`, whose first step runs
`cargo test --workspace`. That executes a standalone default main directly.
It is a real allocation observation, but it supplies neither nextest case
discovery nor assigned-worker evidence.

Only if the real PR/archive flow omits the expected binary or cannot supply
its worker/JUnit receipt, prepare a temporary hosted diagnostic on the exact
source. Reuse the pinned toolchain, nextest and existing Cargo lockfile:

```sh
cargo nextest list --locked --cargo-profile ci --profile full -p vize_patina --test davinci_authored_markup_budget --message-format json > authored-allocation-inventory.json
cargo nextest run --locked --cargo-profile ci --profile full -p vize_patina --test davinci_authored_markup_budget -E 'test(=authored_projection_reuses_ordinary_storage_and_holds_the_facade_budget)' --no-tests=fail
```

Keep the complete source identity, inventory JSON, actual named execution
log and JUnit. These commands compile and run the real allocation target;
the instrumented callback used by the tooling protocol law supplies no
allocation-window runtime credit. No local Cargo build is needed for this
audit. Later rebases require their own exact-source hosted receipts before
protected queue acceptance and actual merge.
