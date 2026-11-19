Three report gates run in series at the end of every PR and queue Check: `Rust product report` / `Rust source report`, then `PR source report`, then `test-report`. Each did a full `actions/checkout` to run one Node script that reads only `NEEDS_JSON`. `test-report` already uses a sparse checkout (#11778).

This PR gives `rust-product-report`, `rust-source-report` and `source-report` a depth-1 sparse checkout of exactly:

- `package.json`;
- the gate script's literal import closure;
- for the Rust reports, the `cache-vp-bootstrap` local action, which reads only `package.json`.

Gate logic, `needs`, permissions, runners and required checks are unchanged.

## Why (measured)

Last 58 successful `pull_request` Check runs (sample in #13364), checkout step only:

| job | checkout p50 | checkout p90 | whole job p50 |
|---|---|---|---|
| `PR source report` | 13 s | 19 s | 0.4 min |
| `Rust product report` | 14 s | 16 s | 0.4 min |
| `test-report` (already sparse) | 2 s | 5 s | 0.3 min |

`PR source report` sits on every run's critical path, after the last product job. `Rust product report` is on it whenever a Rust cohort finishes last. The expected saving is about 11 s per report, so 11–23 s off the tail of the run. This is small. It is a pure checkout change, and the report chain added 0.75 min p50 after the last product job.

## Tests

- New `tests/tooling/report-checkout-closure.test.ts`, one test for each of the three reports:
  - the sparse list equals `package.json`, plus the job's local actions, plus the complete literal import closure of its single gate script (`localImportInputs` from the tooling planner, `complete === true`);
  - the gate module then loads from a scratch tree that holds only the sparse paths.
- `frontend-runner-pool.test.ts` registers `source-report` as a reviewed checkout site (`tests/tooling/support/frontend-checkout-contract.ts`), as it does for `test-report` and `iteration-plan`.
- I ran locally (macOS) all 53 tooling test files that read `pr-source-checks`, `pr-rust-products`, `pr-rust-checks` or these report job ids: 265/279 passed.
  - One hash pin needed the reviewed-checkout exception: `github-workflows-rust-source.test.ts`, which now asserts the sparse checkout and hashes the rest of the report prefix unchanged.
  - 11 failures in `setup-wild-http-recovery.test.mjs` need GNU coreutils, and 1 in `native-vapor-capture-workflow.test.mjs` needs the Linux runtime toolchain. Neither file reads the changed jobs.
  - 1 `frontend-runner-pool` truth-table case hit its 100 ms VM timeout under load; it passed on rerun.

## After

Check [44472765195](https://github.com/ubugeeei-prod/vize/actions/runs/44472765195), on the first head `e81da18`, checkout step only:

| job | before p50 (58 runs) | this run |
|---|---|---|
| `PR source report` | 13 s | **6 s** |
| `Rust product report` | 14 s | **1 s** |
| `Rust source report` | (full checkout) | **5 s** |
| `test-report` (unchanged) | 2 s | 1 s |

That run failed for a reason unrelated to the checkout. The new test file passed an untyped `.mjs` return into a template literal. `check-js` reported this as a lint warning against its zero-warning budget, and `typescript-tooling-project.test.ts` reported it as TS2345. Follow-up commit `f72632e` types the closure. With that commit, `typescript-tooling-project.test.ts` and the closure test pass locally (4/4). The exact-head rerun [44476141807](https://github.com/ubugeeei-prod/vize/actions/runs/44476141807) succeeded. Its checkouts took 1 s for `Rust source report`, 1 s for `Rust product report` and 6 s for `PR source report`; `test-report`, which this PR does not change, took 7 s. The run took 19.5 min overall because runners were queued fleet-wide; the queue, not these jobs, set that time.

## Product binaries

Unchanged. Only workflow YAML and tooling tests change. No file under `crates/`, `davinci/`, `npm/` or `Cargo.*` changes, so the `vize` CLI, NAPI and WASM outputs are byte-identical by construction.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
