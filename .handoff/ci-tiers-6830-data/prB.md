The `vize_canon` integration test `empty_vue_star_exports` rebuilds the frozen inputs by inverting about 50 history layers (`membership/constructor_without_args/payload_update/*`).

- Successor layers re-read, re-hash and re-parse their predecessors' committed JSON records each time they are called.
- `public_layout::read` recomputes every path's whole chain on each call.
- So one test re-walks the same deterministic chain many times, in an unoptimized `ci`-profile binary. Each new fix layer adds to the cost.

**Change (test code only):**

- The 15 argument-free record loaders each authenticate their record once per process (`OnceLock`) and return a clone.
- `payload_update::source_before(path)` computes each path's pre-history once per process. The lock is not held while computing, because layers can reach other paths.

Every assertion still executes, once instead of once per caller. Inputs, records, expected bytes and hashes are unchanged.

## Why (measured)

Last 58 successful `pull_request` Check runs (same sample as #13364):

- `Rust tests (Type checker)` ran 6.4 min p50 and 8.3 min p90. It was the last product job in 4/58 runs, and in 22/58 once the native lanes leave the PR (#13364).
- In job [135312670727](https://github.com/ubugeeei-prod/vize/actions/runs/44450333316/job/135312670727), nextest took 238.6 s. Two tests set the tail: `authentic_six_counterfactual_packets_and_provider_bytes_remain_complete` 187.1 s and `configured_native_refuses_erasure_at_unproved_source_boundaries` 193.0 s. The next ones were also this binary's chain: `slot_factory_union_postimage…` 89.2 s, `constructor_postimage_rejects…` 86.1 s and `changed_body_reference_order…` 50.8 s. Every other test took ≤ 26 s.
- On 2026-10-19 the same cohort took 2.5 min p50.

**Local CPU** for `authentic_six_counterfactual_packets_and_provider_bytes_remain_complete`, ci profile. The machine's load average was 187 on 15 cores, so I compare user+sys CPU, not wall time:

| build | user | sys | CPU total | result |
|---|---|---|---|---|
| base `3c1e045563` | 156.4 s | 61.3 s | **≥ 217.6 s** | stopped unfinished after 17 min wall |
| head | 19.7 s | 6.6 s | **26.3 s** | `ok. 1 passed` |

## After (CI, same runner class)

Exact-head Check [44465767690](https://github.com/ubugeeei-prod/vize/actions/runs/44465767690), `Rust tests (Type checker)` job [135363314135](https://github.com/ubugeeei-prod/vize/actions/runs/44465767690/job/135363314135), compared with base job [135312670727](https://github.com/ubugeeei-prod/vize/actions/runs/44450333316/job/135312670727):

| | base | head |
|---|---|---|
| job wall | 6m 37s | **4m 51s** |
| cargo build (`Finished ci profile`) | 1m 34s | 1m 35s |
| nextest summary | 238.551 s | **141.468 s** |
| tests run / passed / skipped | 4717 / 4717 / 843 | 4717 / 4717 / 843 |
| `authentic_six_counterfactual_packets_and_provider_bytes_remain_complete` | 187.097 s | **15.423 s** |
| `configured_native_refuses_erasure_at_unproved_source_boundaries` (tsgo required) | 193.041 s | **27.413 s** |
| `membership::current_updates::slot_factory_union_postimage_rejects_unrelated_or_missing_bytes` | 89.202 s | 11.950 s |
| `membership::constructor_without_args::constructor_postimage_rejects_missing_duplicate_unknown_and_extra_bytes` | 86.089 s | 11.112 s |
| `…public_instance::tests::changed_body_reference_order_missing_duplicate_and_…` | 50.816 s | 17.667 s |

The same test inventory ran with `VIZE_TEST_REQUIRE_TSGO=1` and passed. The whole Check took 13.7 min, but runner queueing dominated it: the Rust cohorts waited 2.4–4.1 min for a runner and the tooling shards 6.1 min (p50). The work itself did not set the run time. The cohort's slowest test is now a 27 s tsgo law; the old tail was two tests of about 190 s.

## Product binaries

Unchanged. Only files under `crates/vize_canon/tests/empty_vue_star_exports/` change. That is an integration-test target, not linked into any library, CLI, NAPI or WASM artifact. The `vize` binary is byte-identical by construction.

## Checks run locally

- `rust-script tools/commands/davinci/assertion-lint.rs`: OK. No assertion was removed.
- `rustfmt --check` on the changed files: clean.
- `source-file-lengths.ts`: exit 0. The largest touched file has 320 lines.
- The whole `empty_vue_star_exports` binary ran in CI with tsgo required: all 4717 cohort tests passed (above).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
