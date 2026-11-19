The maintainer decided on 2026-11-19 (#6830) to keep the Blacksmith pool size (about 40 concurrent jobs) and cut jobs per PR instead. This is the first step of the redesign record added here, `docs/davinci/decisions/2026-11-19-6830-pr-check-redesign.md`.

Seven Check jobs (eight with the node-engine matrix) print only a skip notice when their plan selects nothing, yet each held a 32-vCPU Blacksmith slot. Each now uses a planner-conditional `runs-on`: Blacksmith when its plan selects work, hosted `ubuntu-24.04` when it can only skip.

| job | condition for Blacksmith |
|---|---|
| `security-audit` | `needs.iteration-plan.outputs.dependencies == 'true'` |
| `node-engine-compat (22/24)` | `needs.iteration-plan.outputs.engines == 'true'` |
| `check-vize-apps` | `needs.iteration-plan.outputs.apps == 'true'` |
| `WASM release build check` | `contains(fromJSON(needs.pr-source-plan.outputs.rust-plan).packages, 'vize_vitrine')` |
| `CI selection tests` | `needs.pr-source-plan.outputs.tooling == 'true'` |
| `Native runtime (…)` | `matrix.lane != 'none'` |
| `PR playground browser tests` | `needs.pr-source-plan.outputs.playground == 'true'` |

Each condition is the one the job's own steps already test, so selection, steps, `needs`, required contexts and the merge queue are unchanged. On merge candidates, every selected job still gets the maximum runner.

## Why (measured)

- **Last 58 successful PR Checks (#13364 sample).** A run used 32–75 Blacksmith jobs, and 6.4 of them per run lasted under 25 s. Those were the skip-only paths: playground 58/58 runs, `security-audit` 50, `node-engine-compat` 37 + 37, CI selection 17.
- **5 PR runs after #13364/#13368** (44463861344, 44465767690, 44466029920, 44466877966, 44468797349). `Native runtime (none)` waited 9.2 min (p50) in the Blacksmith queue to print one line, and `PR source report` waits for it. The playground skip waited 8.9 min.

## Tests

- New `tests/tooling/support/planned-runner.ts` lists the reviewed job → condition pairs.
- `workflow-layout.test.ts` accepts only those exact pairs as the maximum runner. A new test rejects the same expression in another workflow, or with another condition.
- `frontend-runner-pool.test.ts` restores the planned runner to the reviewed maximum before comparing against the original job contracts.
- `vp-bootstrap-cache.test.mjs` expects the planned runner for the dependency consumers.
- Locally I ran the 86 tooling test files that read `check.yml`/`pr-source-checks.yml` or these jobs: 481 pass, 15 fail, 3 skip.
  - 11 failures are `setup-wild-http-recovery` cases that need GNU coreutils.
  - 1 is `native-vapor-capture-workflow`, which needs the Linux toolchain.
  - 2 are `full-rust-runtime` `spawnSync … ETIMEDOUT` under load; both pass on rerun (11/11).
  - The last is `vp-bootstrap-cache`, fixed in this PR (8/8).

## After

Exact-head Check [44490027837](https://github.com/ubugeeei-prod/vize/actions/runs/44490027837) succeeded. This PR changes workflows, so its plan selected dependencies, engines, apps, CI selection and WASM, and those jobs correctly took Blacksmith. Two jobs had nothing selected and ran hosted:

| job | runner | queue | run |
|---|---|---|---|
| `Native runtime (none)` | ubuntu-24.04 | 2 s | 4 s |
| `PR playground browser tests` (skip) | ubuntu-24.04 | 3 s | 9 s |

Before, in the five post-#13364 runs, these two waited 9.2 min and 8.9 min (p50) for a Blacksmith slot. The whole run took 37 min: fleet-wide queueing (the Blacksmith-selected jobs above waited 5–7 min), not this change.

## Product binaries

Unchanged. Only workflow YAML, tooling tests and a decision record change; no file under `crates/`, `davinci/`, `npm/` or `Cargo.*`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
