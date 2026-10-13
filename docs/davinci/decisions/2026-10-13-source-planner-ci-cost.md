# CI cost audit, 2026-10-13

Read-only measurements collected around 04:10–04:20 UTC. Source policy pinned to `b45d789b317933d48dadbba4841c8e353231547b`; no rerun, workflow mutation or gate reduction performed. `ubugeeei-redundancy.md` was reread. Preserve original tests, diagnostics, public receipts and full event/caller gates.

## Selected implementation

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Use the same SHA-pinned setup-node v6 as #8810 only in `pr-source-plan`,
with `node-version-file: package.json` and `package-manager-cache: false`.
The planner and inventory graph uses Node builtins, local modules and Cargo
metadata. Preserve every command, output, event condition, dependency edge,
mandatory preflight, runner and five-minute deadline. Existing workflow laws
now reject a package-manager bootstrap while retaining failure propagation.
Keep the real Vite+/Vue/TSGO installs in downstream test consumers.

Local Node25.8.1 checks: the new runtime contract fails on the unchanged parent
workflow;22 source-selection/preflight/plan/dependency regressions pass after
the change. Hosted Node24.14 validation remains required.

The parent measured144s setup out of180s planning. Fresh child exact-head
Actions must prove the existing inventories/plans still pass and measure the
new setup and total planner duration; this is an expected improvement, not
an achieved whole-CI speed claim. Native Stack, exact-head gates and protected
queue delivery remain required. Larger cache/build candidates below remain
unimplemented and require their own bounded measurements.

## Exact run evidence

| Event / run | Head | Result | Measured wall time / limitation |
| --- | --- | --- | --- |
| [PR Check 38341424403](https://github.com/ubugeeei-prod/vize/actions/runs/38341424403) | `d64adbc0085232ad11778838bfbf20dfd3bded3c` | success | 03:22:46→04:06:55, 44m09s; different changes from queue sample |
| [merge_group Check 38340776641](https://github.com/ubugeeei-prod/vize/actions/runs/38340776641) | `b45d789b317933d48dadbba4841c8e353231547b` | success | 03:14:44→03:33:38, 18m54s; required source tier, not full schedule/dispatch qualification |
| [main push Check 38342328555](https://github.com/ubugeeei-prod/vize/actions/runs/38342328555) | same `b45d789…` | in progress at measurement | Starts03:34:11; main push excludes Nix/full workspace/coverage/editor/parity suites by policy, cannot be called full Check |
| [last scheduled main full Check 38237754010](https://github.com/ubugeeei-prod/vize/actions/runs/38237754010) | `01fb9eaa1089a6d5175eb0f9dd88df6285ebbd19` | failure | 10:40:03→11:14:09 on Oct12, 34m06s; older SHA, Nix failed and scripts hit30m deadline |
| [previous main push 38341999006](https://github.com/ubugeeei-prod/vize/actions/runs/38341999006) | `6fb01f335952780d7a6321ca092049753645c83d` | cancelled | New main push created03:34:11; `check-v3-Check-refs/heads/main`, cancel-in-progress:true allows supersession. Cancellation is not test failure. Explicit dispatch/schedule uses `full-{sha}` and is insulated. |

Successor repair [#8810](https://github.com/ubugeeei-prod/vize/pull/8810) exact
`d386fec942a26e190856d02050162cb64d0bd758` level direction
[job115124107084](https://github.com/ubugeeei-prod/vize/actions/runs/38344961197/job/115124107084)
was independently verified success04:07:57→04:08:21 UTC,24s, against main
job115117177368 duration301s (setup-vp296s,cancelled). This is evidence for
the bounded Node-only level gate repair, not whole-PR success. Its source
planner [job115124107148](https://github.com/ubugeeei-prod/vize/actions/runs/38344961197/job/115124107148)
ran180s04:07:57→04:10:57, including setup-vp144s04:08:02→04:10:26.
Original remaining commands took33s, including inventory21s.

## DAG and dominant costs

PR plan job115117975618 started03:42:57, **20m11s after run creation**, then ran52s. Downstream producer/source jobs could not start before03:43:56. This scheduler wait alone is45.7% of44m09s. The timestamp delta measures wait, not proven runner-capacity cause.

PR Rust chain: build115118206555 ran335s; shard3 115119603586 ran926s; source report11512340039825s; source report11512351659914s; top test report11512358837740s; PR comment11512377246423s. **Shard3 setup-vp742s vs actual nextest109s**. Log: installing vite-plus1.1.0 started03:49:51; next emitted installing dependencies04:00:24 (~633s bootstrap), dependencies20s, then Node24.14/cache598MB restore and vp install~79s (1915 reused,0 downloaded). No explicit retry/error explains the633s silence, so do not claim a diagnosed network endpoint failure. Other Rust shard setup-vp durations219/257/409s; archive download49–53s; tests83–109s.

Sum of successful job wall durations: PR11,324s (~188.7 runner minutes), queue8,267s (~137.8 runner minutes). These are sums of elapsed jobs, not billable CPU or normalized performance comparison. Individually visible setup-vp steps: PR4,634s (~77.2 runner minutes), queue185s (~3.1 runner minutes); composites may hide additional setup time.

Queue critical paths: source CLI producer11511124644786s (build57s) unlocks tooling shard1 115111634380895s, including tooling611s plus current native evidence; required Rust differential115112525310679s (corpora550s) follows full archive producer115111246358298s. JS115111246227911s (package tests726s, UI147s), reach115111246325831s (actual reach708s), ssr-pug115111246345565s (corpora435s). Rust shards177–197s, actual nextest97–118s; archive transfer44–47s. More Rust shards alone will not improve the18m54s queue critical path.

Main push `build-js-packages`1151171774261515s: setup-vp245s, actual native/UI268s, **post sticky+Actions cache907s**. Log tar/zstd starts03:49:46.537; target seed saved04:04:51.504 (~905s). `check-vize-apps`115117177638585s includes build42s and post-cache394s. Sticky mounts succeeded. New fingerprint seed was absent, so this is one-time-per-lock/toolchain/role Actions seed serialization, not proof that every main SHA rearchives. API query did not expose this entry's size; do not invent GB figures. Existing source already suppresses target saves for clippy-test/coverage-source after historical25GB backend limits.

Last full scheduled run: source coverage1147725301532037s, source report1976s (critical path); full clippy-and-test1147725301701617s, Clippy138s/allocation212s/Test1202s; scripts1147725300771808s (cancelled), test release scripts1483s; full JS114772530121924s with tests894s. These older timings identify targets, not current-main qualification. Full suite cannot be replaced by queue history.

## Rust build/cache identity

Committed toolchain is1.99.0; PR/queue logs confirm rustc1.99.0(b940084d7,2026-09-28). PR builder pins1.99.0; several full jobs use stable; semver intentionally1.95.0, branch coverage nightly. Lock/toolchain fingerprint is`1ea6087ed98bd6cc1048efef91faa0112497b52eead3d2bbdbc26fe9effff72a` in these logs. Cache policy fingerprints committed Cargo.lock/rust-toolchain.toml, not the live stable/nightly compiler output; actual toolchain identity needs care when designing alternate seed roles.

PR builder115118206555 had registry/git hits but `nextest-ci-target` miss. Queue producer115111246358 had registry/git hits but `clippy-test-target` miss. Sticky is allowed only trusted main push/schedule/dispatch, never PR/merge_group. `nextest-ci` appears only in the untrusted PR producer, so there is no current trusted producer for that role. `clippy-test` deliberately restore-only due historic entry size. Avoid opening sticky mutable targets to PRs to solve these misses.

Archive build-once design already exists: PR cargo nextest archive affected packages117s, upload33s; queue full workspace archive123s, upload31s; four consumers run --archive-file without rebuilding. Receipt verifies SHA/tree/platform/rustc/profile/nextest/runtime envelope. Doctests execute separately (queue46s). Source CLI producer separately builds `cargo build --profile ci -p vize` plus SFC oracle, shares receipt-bound binaries among four tooling consumers; apps still builds own CLI47s in queue. Independent profile/features/platform artifacts and original-build receipt requirements prevent treating all builds as interchangeable.

## Bounded next actions, preserving all gates

1. **Fix runtime bootstrap at the actual dependency boundary.** #8810 removes Vite+ only from a proved Node-only level job;24s successor vs301s cancelled main confirms this narrow repair. Extend direct Node only to similarly dependency-free planning/reporting actions with unchanged scripts/gates. Investigate633s Vite+ bootstrap gap separately; consumers that need packages must retain full installation and real Vue/TSGO runtime. Compare same exact head/event at least twice, separate bootstrap/cache/installation/execution and preserve failing receipts.
2. **Reduce trusted seed serialization cost without weakening isolation.** Current role-specific whole-target tar cost905s warrants a bounded compact build-cache/seed experiment; keep sticky for trusted main and restore-only for PR/queue. Measure size/serialization/restore plus correctness before accepting a trimmed Cargo seed. Fingerprint/profile/toolchain/runtime must remain explicit; don't add background work that races unmount or hides failure. Source target roles already specially suppress excessive archives; candidate must preserve existing actual-build evidence and every gate.
3. **Measure full-suite execution and cold Rust roles before changing build systems.** The full current path runs `cargo test --workspace` plus feature corpora serially while queue already uses verified nextest build-once archives. Evaluate full nextest archive/shards only with same registered test inventory, full TSGO fixture envelopes, doctests, all feature corpora, allocation/Nix/coverage/editor/native/public receipts preserved; instrumented coverage retains separate compiler products. Also quantify a trusted compact nextest-ci seed because current role is never populated. Changes are candidates, not proof of savings; no Bazel/Ninja/dependency migration is supported by the measured primary costs.

No direct-Node Rust shard change is safe merely because its visible command is cargo: setup-vp `run-install:true` populates real workspace dependencies. `crates/vize_canon/tests/original_jsx_check.rs:80` unconditionally reads actual `docs/node_modules/vue/package.json` version3.5.35 and Vue runtime typings; `crates/vize/tests/check_instance_props_cli.rs:31` resolves actual node_modules/.bin/tsgo and Vue. Merge envelope requires VIZE_TEST_REQUIRE_TSGO=1. The worker also uses `vp node tests/differential/typechecker-shards.ts`. Removing setup/install would break or silently skip original acceptance, so no shard child was created. The separate Rust source report is dependency-free (Node builtins/local imports, install:false), but costs25s and cannot resolve the742s shard bootstrap. A report-only child would be modest overhead hardening, not the main CI acceleration.

## Reproduction commands

Retain raw JSON and subtract startedAt/completedAt for each job/step:

```sh
rtk gh run view 38341424403 --json jobs,headSha,event,status,conclusion
rtk gh run view 38340776641 --json jobs,headSha,event,status,conclusion
rtk gh run view 38237754010 --json jobs,headSha,event,status,conclusion
rtk gh run view 38341424403 --job 115119603586 --log
rtk gh run view 38340776641 --job 115111246358 --log
rtk gh api repos/ubugeeei-prod/vize/actions/jobs/115117177426/logs --allow-escape-sequences
```

Pin workflow/cache policy with `git show b45d789b317933d48dadbba4841c8e353231547b:<path>`;
avoid mutable origin/main when reproducing this snapshot.
