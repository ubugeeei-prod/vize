# Stop owned Rust compiler processes before cache teardown

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed delay

The replacement PR Check run
[38893209547](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547)
at `986d6cdb14d866bf23786099acbc4f57dff5c824` was created at 11:21:16 UTC.
Its first jobs started at 11:23:40, 144 seconds later. The previous run
[38892945040](https://github.com/ubugeeei-prod/vize/actions/runs/38892945040)
at `fccbc44b6631c04de00153cd1a87258f1d0733e9` did not finish cancellation
until 11:23:39. Both PR runs belong to the same cancel-in-progress group.

The previous
[Compiler cohort](https://github.com/ubugeeei-prod/vize/actions/runs/38892945040/job/116946725992)
was the last cancelled worker. Its step reported cancellation at 11:21:35.754,
but Cargo, cargo-nextest, rustc and linker descendants survived until the
runner's final orphan cleanup at 11:23:28.902–11:23:29.054. Its sticky-cache
post actions occupied 113 seconds between those events:

| Post action         |  Elapsed | Observed cause                                                                                                       |
| ------------------- | -------: | -------------------------------------------------------------------------------------------------------------------- |
| Target disk         | 19.319 s | 12.338 s from mounted-device detection to usage report, then ten failed unmount attempts because the target was busy |
| Cargo git disk      | 48.502 s | 47.084 s before usage report; successful unmount; explicitly no commit                                               |
| Cargo registry disk | 44.770 s | 44.113 s before usage report; successful unmount; explicitly no commit                                               |

The pinned `useblacksmith/stickydisk` version
[`3f6be1451e3cad893b778bc93495c80452a5da26`](https://github.com/useblacksmith/stickydisk/blob/3f6be1451e3cad893b778bc93495c80452a5da26/src/post.ts)
runs global `sync`, then `df`, cache dropping and unmount before checking its
read-only/no-commit setting. The three intervals before usage reporting total
103.535 seconds. Existing timestamps do not separate `sync` from `df`, and do
not prove that a particular process was in uninterruptible kernel sleep.
The logs do prove that this was teardown with live compiler descendants,
not a cache upload or commit.

The replacement runner's own Blacksmith message-wait and acquisition timings
were 747 ms and 358 ms. It started one second after the old run became terminal;
the evidence does not support attributing the 144-second wait to runner capacity.

## Change

The first hosted cancellation probe,
[38896572920](https://github.com/ubugeeei-prod/vize/actions/runs/38896572920),
exposed an additional boundary: no owned-group message appeared, target unmount
was busy, and nextest/test processes survived until orphan cleanup. GitHub
runner 2.337.0 sends its initial Unix signal to the
[launched process PID](https://github.com/actions/runner/blob/v2.337.0/src/Runner.Sdk/ProcessInvoker.cs),
which was Bash waiting for Node. The original local fixture signalled the Node
descendant directly and therefore did not validate that boundary. Signalling
the actual Bash entry reproduced the failure locally: it remained running
after six seconds.

The PR step now uses `exec node`, so its original PID becomes the typed runner.
The runner writes the existing `phase`, `cohort`, `elapsed_seconds` and
`exit_code` timing receipt when `VIZE_RUST_COHORT_TIMING_PATH` is explicitly set.
This preserves failure/cancellation status without a waiting shell in front of
the signal handler. Its normal Cargo arguments are unchanged.

The cohort CLI previously used `spawnSync` for `cargo nextest run`. Its Node
event loop could not handle cancellation while Cargo was running. Launch that
same executable and argument vector asynchronously in a new POSIX process
group, with inherited environment, working directory and standard streams.

On SIGINT or SIGTERM, signal only that owned group. If it remains alive,
send a second SIGTERM after one second, including when the first signal was
SIGTERM. Nextest owns separate test groups; its pinned
[0.9.146 shutdown implementation](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/nextest-runner/src/runner/unix.rs)
uses that second signal to kill those groups itself. Give nextest time to reap
them before sending SIGKILL to its group at four seconds if still necessary.
Await the direct child's completion and the disappearance of
the group, including grandchildren remaining after Cargo itself exits, before
returning. The overall drain limit is six seconds from the first signal,
within the Actions runner's initial 7.5-second grace period.
An undrained group produces an explicit failure with exit status 1; it is
never reported as clean.
SIGKILL cannot promise immediate completion of uninterruptible kernel I/O.

Cancellation returns 130 for SIGINT or 143 for SIGTERM, including when a child
catches the signal and exits zero. Ordinary exit statuses and spawn errors
remain unchanged; an independently signalled child still returns the previous
nonzero status of 1. This cannot turn a cancelled Rust step into a passing
required report.

The ownership boundary is the process group created by this invocation. No
process-name scan, host-wide kill, cache-policy exception or post-action bypass
is introduced. A process that deliberately creates a different session/group
is outside this boundary. The affected package plan, test-target projection,
features, nextest options, PR/merge-queue roots and job concurrency are unchanged.
The short existing metadata command is also unchanged.

## Validation and remaining measurement

`rust-test-cohort-cancellation.test.ts` executes the real CLI with a temporary
Cargo provider and real Node child/grandchild processes. It checks inherited
argv/environment/cwd/output, ordinary success/failure, independent child signal
failure and spawn failure. Cancellation controls cover SIGINT, SIGTERM, ignored
SIGINT, a leader exiting zero before its grandchild, and descendants ignoring
both graceful signals. Every control requires the owned group to be gone before
the CLI exits while an unrelated process remains alive.

Every cancellation control runs the actual PR workflow shell and signals its
entry PID, checking that `exec` replaced Bash with the Node runner. Its timing
receipt and exit status must retain 130/143, and both failed and cancelled
cohort results must fail the required Rust aggregate. Local validation uses
macOS with Node 26.12.0 and the pinned Node 24.14.0. The hosted experiment below
records Linux teardown evidence separately from those local process controls.

The opt-in `rust-test-cohort-nextest-cancellation.test.ts` requires installed
Rust and exactly cargo-nextest 0.9.146. It builds only a dependency-free
temporary crate, whose test ignores SIGINT/SIGTERM and starts a Node descendant.
The control proves that the test has a different process group from nextest,
signals the real workflow entry, and requires both groups and the descendant
to disappear before the runner exits. It does not build the repository or
download/install tools; explicitly enabling it with missing tools fails.

With the previous escalation, initial SIGTERM killed nextest at 2.041 seconds
while the test and descendant remained alive. After always sending the second
signal, initial SIGINT completed in 1.049 seconds and SIGTERM in 1.044 seconds;
both logged nextest's second-signal cleanup and left no observed survivors.
This local isolated probe used Rust 1.99.0 on macOS; it is not qualification of
the current product workspace or its subsequently updated Rust toolchain.
The downloaded macOS 0.9.146 archive matched the upstream SHA-256
`39785160b3c2f6ed9a765049cf4fa79f3b39aa02eb7598a5a0e2a1a0b9ffb9a8`.

To repeat using the already installed pinned nextest and Rust on `PATH`:

```sh
VIZE_CANCELLATION_REAL_NEXTEST=1 node --test tests/tooling/rust-test-cohort-nextest-cancellation.test.ts
```

Existing cohort-projection, workflow and strict Rust-gate tests remain required.
Fresh Actions on the committed change must establish ordinary CI acceptance;
a controlled cancellation of our own run can measure hosted drain latency.
Local process controls alone do not establish that sticky-cache teardown or
the PR duration target has been achieved.

The observed replacement run reached its required report in 8m02s from run
creation, or 5m38s from first job allocation. Removing the cancellation wait
alone therefore would not meet the five-minute PR target. Keep the actual
creation-to-required-report measurement, including queue time.

## Hosted cancellation proof

The second controlled experiment cancelled only [Check 38899025329, attempt 1](https://github.com/ubugeeei-prod/vize/actions/runs/38899025329/attempts/1), at source head `986df9a534e18941da8d23cb90ed2bf8d733f355`, base `d6c82266aec2d44eff688c679ce38959d882a2e4`, and tested merge `2b10e767adecca793bcc47918692bf3f5b5ba9e8`. The recorded command boundary was `2026-10-16T12:16:35.857Z` immediately before `gh run cancel` and `12:16:37.687Z` immediately afterward; the command accepted the request.

Both the [Compiler job](https://github.com/ubugeeei-prod/vize/actions/runs/38899025329/job/116966508517) and [Language server job](https://github.com/ubugeeei-prod/vize/actions/runs/38899025329/job/116966508578) were executing nextest when they logged `Rust cohort received SIGINT; stopping its owned Cargo group.` Compiler received it at `12:16:52.1398095Z`, Language server at `12:16:51.9215235Z`. This is 14.452 s and 14.234 s after the cancellation command returned; external request delivery is distinct from the handler's cleanup time. Their jobs completed 4.861 s and 3.079 s after the handler message.

| Cohort               | Outcome                               | Rust cache post step | Target unmount action | Logged orphan terminations | Busy or failed unmounts |
| -------------------- | ------------------------------------- | -------------------: | --------------------: | -------------------------: | ----------------------: |
| Compiler             | Cancelled during tests                |                  3 s |               3.106 s |                          0 |                       0 |
| Language server      | Cancelled during tests                |                  1 s |               1.536 s |                          0 |                       0 |
| Type checker         | Tests passed; job cancelled afterward |                  3 s |               2.941 s |                          0 |                       0 |
| Lint and format      | Passed before request                 |                  4 s |               2.992 s |                          0 |                       0 |
| CLI and integrations | Passed before request                 |                  3 s |               3.171 s |                          0 |                       0 |

All 15 target/git/registry mounts unmounted successfully. Compared with the earlier controlled run `38896572920`, the Compiler cache post step changed from 34 s with 34 logged orphan terminations and a busy target unmount to 3 s with none. Language server changed from 5 s with 24 logged orphan terminations and a busy target unmount to 1 s with none. These are observed cross-run differences, with different source revisions and cancellation positions.

Nextest reacted to the initial SIGINT and the runner reported cancellation within 100 ms in both active cohorts. The one-second second-signal fallback was not exercised by this hosted run. Compiler and Language server artifact upload was skipped on cancellation, so an actual process exit 130/143 was not observable; do not substitute the implementation's expected code for a retained result. No Linux process-state snapshot was captured. Zero logged orphan terminations and successful unmounts are the available cleanup evidence.

The required `test-report` refused `pr-source-checks: cancelled` and exited 1. The intentionally cancelled workflow completed after 235 s and consumed 46.8 runner-minutes. This is an expected non-green cancellation experiment, not evidence of ordinary PR latency or the five-minute target. A full same-head rerun must be recorded separately.

## Complete normal rerun

[Attempt 2](https://github.com/ubugeeei-prod/vize/actions/runs/38899025329/attempts/2)
reran all 33 jobs at the same source head, reusing zero completed jobs. From
the attempt start at 12:19:06 UTC to the required report at 12:23:15, it took
249 seconds (4m09s), consuming 44.8 runner-minutes. Clippy, all 18,159 selected
Rust tests, every native runtime lane and the strict reports passed. The five
cohorts contained no duplicate test identity and every timing receipt recorded
exit code zero. Compiler retained the same 7,154 identities and Language server
the same 1,701 as the earlier PR measurement. Fresh main adds 20 other identities
and replaces two Patina names, for a net increase of 18 tests.

The critical path is now JS builders: 208 seconds, including 24 seconds of setup
and a 168-second product step. Native Cargo accounts for 46.95 seconds inside
that step; serial package build/test work remains a measured follow-up target.
Compiler took 193 seconds (Cargo 57.79; nextest 94.342), Language server 175,
and the combined Clippy/navigation/doctest lane 159. All ordinary sibling
workflows at the same head passed; Musea was longest at 230 seconds.

This is a complete normal rerun after an intentional cancellation, not a fresh
PR push or a controlled attribution experiment. Original workflow creation to
the final required report spans 588 seconds, including the cancelled attempt
and time before retry; all ordinary workflows span 592 seconds. The measured
4m09s normal attempt meets the five-minute target for this observation. It is
not a guarantee for future queue or cold-cache conditions, and it does not
establish final App qualification or end-to-end release time.
