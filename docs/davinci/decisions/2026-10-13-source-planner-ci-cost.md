# Direct Node setup for source planning and measured CI costs

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Measurements were collected on 2026-10-13 around 04:10–04:20 UTC. Workflow
policy is pinned to `b45d789b317933d48dadbba4841c8e353231547b`.
`ubugeeei-redundancy.md` was reread. The audit did not rerun workflows or
change gates, caches, dependencies or build systems.

## Selected change

Use the same SHA-pinned setup-node v6 as #8810 only in `pr-source-plan`,
with `node-version-file: package.json` and `package-manager-cache: false`.
The planner and inventory graph uses Node built-ins, local modules and Cargo
metadata. Every original command, output, event condition, dependency edge,
mandatory preflight, runner and five-minute deadline stays intact. Existing
workflow regressions reject package-manager bootstrap while retaining required
preflight failure propagation. Downstream test consumers keep their real
Vite+/Vue/tsgo dependencies.

The parent planner spent 144 of 182 seconds in runtime setup. Local Node
25.8.1 validation: the new runtime contract fails on the parent workflow;
22 source-selection, preflight, planning and dependency regressions pass after
the change. The two actual planner preflight files also pass all eight tests
on the declared Node 24.14 runtime without installed workspace dependencies.
A byte comparison confirms only the planner setup action and its inputs changed.
Fresh child Actions must prove all existing inventories/plans and measure the
new setup and total planner duration. Native Stack and protected queue delivery
remain required. Larger candidates below remain unimplemented.

## Exact run evidence

| Event / run                                                                                             | Head                                       | Result                    | Wall time and limitation                                                                              |
| ------------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------- | ----------------------------------------------------------------------------------------------------- |
| [PR Check 38341424403](https://github.com/ubugeeei-prod/vize/actions/runs/38341424403)                  | `d64adbc0085232ad11778838bfbf20dfd3bded3c` | success                   | 03:22:46–04:06:55, 44m 09s; different changes from queue sample                                       |
| [Merge group Check 38340776641](https://github.com/ubugeeei-prod/vize/actions/runs/38340776641)         | `b45d789b317933d48dadbba4841c8e353231547b` | success                   | 03:14:44–03:33:38, 18m 54s; required source tier                                                      |
| [Main push Check 38342328555](https://github.com/ubugeeei-prod/vize/actions/runs/38342328555)           | same `b45d789…`                            | in progress when measured | Starts 03:34:11; main push excludes full workspace, Nix, coverage, editor and parity suites by policy |
| [Last scheduled full Check 38237754010](https://github.com/ubugeeei-prod/vize/actions/runs/38237754010) | `01fb9eaa1089a6d5175eb0f9dd88df6285ebbd19` | failure                   | Oct 12, 10:40:03–11:14:09, 34m 06s; older SHA, Nix failed and scripts hit their deadline              |
| [Previous main push 38341999006](https://github.com/ubugeeei-prod/vize/actions/runs/38341999006)        | `6fb01f335952780d7a6321ca092049753645c83d` | cancelled                 | New main push created at 03:34:11; permitted supersession, not a test failure                         |

Main push concurrency uses `check-v3-Check-refs/heads/main` with
`cancel-in-progress: true`. Explicit dispatch and schedule use `full-{sha}`
and are insulated from main push supersession. Neither queue source success
nor main push success proves the full schedule/dispatch qualification.

Parent [#8810](https://github.com/ubugeeei-prod/vize/pull/8810) at exact head
`d386fec942a26e190856d02050162cb64d0bd758` supplies a narrow comparison:

| Job                                                                                                                   | Duration                | Observed phase                                                                   |
| --------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------------------------------------------------------------------------------- |
| [Dependency direction 115124107084](https://github.com/ubugeeei-prod/vize/actions/runs/38344961197/job/115124107084)  | 24s, 04:07:57–04:08:21  | Direct Node successor succeeded                                                  |
| Main dependency direction 115117177368                                                                                | 301s                    | setup-vp consumed 296s and was cancelled                                         |
| [Parent source planner 115124107148](https://github.com/ubugeeei-prod/vize/actions/runs/38344961197/job/115124107148) | 182s, 04:07:57–04:10:59 | setup-vp 144s, 04:08:02–04:10:26; remaining commands 33s including inventory 21s |

The 24s dependency result proves the bounded level-gate repair, not whole-PR
success or universal runtime-download availability.

## DAG and dominant costs

PR planner job 115117975618 started at 03:42:57, **20m 11s after run creation**,
then ran for 52s. Its downstream producers could not start before 03:43:56.
This timestamp gap accounts for 45.7% of the 44m 09s run. It measures waiting;
it does not establish a runner-capacity or concurrency-limit cause.

| PR job in run 38341424403  | Total    | Relevant phase                                               |
| -------------------------- | -------- | ------------------------------------------------------------ |
| Rust producer 115118206555 | 335s     | Archive build 117s; upload 33s                               |
| Rust shard 3, 115119603586 | 926s     | **setup-vp 742s**; archive download 53s; nextest 109s        |
| Other Rust shards          | 374–581s | setup-vp 219 / 257 / 409s; downloads 49–51s; nextest 83–108s |
| Rust report 115123400398   | 25s      | Follows all shards                                           |
| Source report 115123516599 | 14s      | Follows complete source DAG                                  |
| Final report 115123588377  | 40s      | Required aggregation                                         |
| PR comment 115123772464    | 23s      | Final completion at 04:06:55                                 |

Shard 3 logged `installing vite-plus@1.1.0` at 03:49:51, then
`Installing dependencies` at 04:00:24: about 633s of bootstrap. Those
dependencies took 20s. Node 24.14 and a 598 MB cache restore followed, then
`vp install` took about 79s, reusing 1,915 packages and downloading zero.
The 633s gap has no explicit retry/error identifying its endpoint or cause.
Actual test execution was 109s; a slow setup must not be described as slow
Rust execution.

| Merge group job in run 38340776641 | Total    | Relevant phase                             |
| ---------------------------------- | -------- | ------------------------------------------ |
| Shared source CLI 115111246447     | 86s      | Build 57s; unlocks tooling consumers       |
| Tooling shard 1, 115111634380      | 895s     | Tooling 611s plus original native evidence |
| Full Rust producer 115111246358    | 298s     | Archive 123s; doctests 46s; upload 31s     |
| Required differential 115112525310 | 679s     | Corpora 550s, follows producer             |
| JS package gate 115111246227       | 911s     | Tests 726s; UI 147s                        |
| Production reach 115111246325      | 831s     | Actual reach 708s                          |
| SSR/Pug 115111246345               | 565s     | Corpora 435s                               |
| Four Rust consumers                | 177–197s | nextest 97–118s; downloads 44–47s          |

Adding Rust shards alone cannot shorten the tooling/differential critical path.
Sum of successful job durations was 11,324s for the PR and 8,267s for the
queue. Individually visible setup-vp steps summed to 4,634s and 185s,
respectively; composites may hide more. These are elapsed job sums, not billed
CPU minutes or a normalized comparison between different source changes.

| Main push job in run 38342328555 | Total  | Relevant phase                                     |
| -------------------------------- | ------ | -------------------------------------------------- |
| Build JS packages 115117177426   | 1,515s | setup-vp 245s; native/UI 268s; **post-cache 907s** |
| Check apps 115117177638          | 585s   | CLI build 42s; post-cache 394s                     |

Sticky mounts worked. The build target seed was absent for the current
lock/toolchain fingerprint. `tar`/`zstd` started at 03:49:46.537; the seed save
finished at 04:04:51.504, about 905s later. This is a one-time Actions seed cost
per lock/toolchain/role, not evidence of rearchiving every main SHA. Cache API
did not expose this entry's size. Do not invent a size from elapsed time.
The implementation already suppresses target saves for `clippy-test` and
`coverage-source` after historical entries exceeded the backend's 25 GB limit.

| Older full scheduled job in run 38237754010 | Total             | Relevant phase                                     |
| ------------------------------------------- | ----------------- | -------------------------------------------------- |
| Source coverage 114772530153                | 2,037s            | Coverage report 1,976s; critical path              |
| Full Clippy/tests 114772530170              | 1,617s            | Clippy 138s; allocation 212s; tests 1,202s         |
| Scripts 114772530077                        | 1,808s, cancelled | Release-script tests 1,483s; original 30m deadline |
| Full JS tests 114772530121                  | 924s              | Tests 894s                                         |

Older full-suite timings identify candidates; they do not qualify current main.

## Rust build and cache identity

Official [stable channel metadata](https://static.rust-lang.org/dist/channel-rust-stable.toml)
was fetched on 2026-10-13: channel date 2026-10-01, Rust/rustc 1.99.0
(`b940084d7`, 2026-09-28), Cargo 0.100.0 (`5f94df478`, 2026-08-27).
Pinned 1.99.0 matches current stable; no toolchain install/update was needed.
PR and queue logs independently confirm rustc 1.99.0. Some full jobs select
`stable`; SemVer intentionally selects 1.95.0 and branch coverage uses nightly.

The cache fingerprint in these logs is
`1ea6087ed98bd6cc1048efef91faa0112497b52eead3d2bbdbc26fe9effff72a`.
It covers committed Cargo.lock and rust-toolchain.toml, not the live
stable/nightly compiler output. Actual compiler identity matters for future seeds.
PR producer 115118206555 had registry/git hits but a `nextest-ci-target` miss.
Queue producer 115111246358 had registry/git hits but a `clippy-test-target` miss.
Only trusted main push/schedule/dispatch can mount sticky disks; PR and merge
checkouts remain restore-only. `nextest-ci` currently appears only in the
untrusted PR producer, so it has no trusted producer. `clippy-test` deliberately
cannot save its whole target. Opening mutable sticky disks to PRs is not a remedy.

Build-once archives already serve four consumers. They bind SHA, tree, platform,
rustc, profile, nextest and runtime envelope through receipts. Consumers execute
`--archive-file` without rebuilding; doctests remain separate. The source CLI
producer separately builds `cargo build --profile ci -p vize` plus the SFC
oracle and shares receipt-bound binaries with four tooling consumers. Apps
still builds its own CLI (47s in the queue sample). Different profiles,
features, platforms and original-build receipt requirements prevent treating
all Rust products as interchangeable.

Rust shards are not dependency-free. Setup-vp `run-install: true` supplies the
real workspace dependencies. `crates/vize_canon/tests/original_jsx_check.rs:80`
unconditionally reads actual `docs/node_modules/vue/package.json` version
3.5.35 and Vue typings. `crates/vize/tests/check_instance_props_cli.rs:31`
resolves actual `node_modules/.bin/tsgo` and Vue. The merge envelope requires
`VIZE_TEST_REQUIRE_TSGO=1`; the worker also invokes
`vp node tests/differential/typechecker-shards.ts`. Removing their setup/install
could break or skip original acceptance, so no Rust shard change was made.

## Bounded follow-ups with every gate retained

1. **Runtime bootstrap:** direct Node belongs only in proven dependency-free
   planning/reporting actions. Investigate the unexplained 633s bootstrap gap
   separately. Package consumers retain complete installation and real Vue/tsgo
   runtime. Compare the same exact head/event, separating setup, installation,
   cache and execution; retain failures and inventories.
2. **Trusted seed cost:** measure a compact role-specific Cargo seed against the
   905s whole-target serialization. Keep trusted sticky disks and restore-only
   PR/queue policy, explicit fingerprint/profile/compiler identity and all
   original-build receipts. Measure size, save, restore and correctness before
   adopting a trimmed seed. No background work may race unmount or hide failure.
3. **Full execution and cold roles:** compare full nextest archive/shards against
   `cargo test --workspace` plus serial feature corpora. Preserve the complete
   registered inventory, required tsgo fixtures, doctests, every feature corpus,
   allocation/Nix/coverage/editor/native/public gate and receipt. Instrumented
   coverage keeps separate compiler products. Measure a trusted compact
   `nextest-ci` seed because that role is never populated today. No Bazel/Ninja
   migration is justified by the primary measured costs.

These are candidates, not demonstrated savings or permission to reduce checks.

## Reproduction

Retain raw JSON and subtract startedAt/completedAt for each job/step:

```sh
rtk gh run view 38341424403 --json jobs,headSha,event,status,conclusion
rtk gh run view 38340776641 --json jobs,headSha,event,status,conclusion
rtk gh run view 38237754010 --json jobs,headSha,event,status,conclusion
rtk gh run view 38341424403 --job 115119603586 --log
rtk gh run view 38340776641 --job 115111246358 --log
rtk gh api repos/ubugeeei-prod/vize/actions/jobs/115117177426/logs --allow-escape-sequences
```

Pin workflow/cache policy with
`git show b45d789b317933d48dadbba4841c8e353231547b:<path>` rather than mutable main.
