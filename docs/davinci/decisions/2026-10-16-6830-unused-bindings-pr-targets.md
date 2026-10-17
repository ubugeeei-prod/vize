# Reuse prepared Rust targets for unused binding checks

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The first passing Check on PR #9200 took 4m42s, but the entire PR was still
waiting for Davinci unused bindings. That workflow passed after 8m11s on
`99caba071ada01a251c3618683fcfe44701ecfa0`. Its job started six seconds after
creation; the delay was inside Cargo compilation, not the runner queue.

| Observation             | Initial `be62f9e` | Updated `99caba0` |
| ----------------------- | ----------------: | ----------------: |
| Workflow                |              245s |              491s |
| Independent specs step  |              133s |              405s |
| Patina test compilation |              101s |              374s |
| CLI compilation         |            40.58s |            40.44s |

Both runs missed `unused-bindings-target-seed` while finding the registry and
git caches. The workflow has no automatic trusted push or schedule writer for
that target namespace. Both also selected floating stable Rust 1.99.1, whereas
the repository and the prepared `nextest-ci` target use the repository pin.
The workflow and relevant product sources did not change between these runs.
The logs isolate the varying wall time to the Patina compilation; they do not
prove which host resource caused the difference between the two cache misses.

## Retained checks

The filtered Patina command compiled 96 integration targets, then ran zero
tests in each. Its six matching tests all belong to the library's
`rules::facts::unused_setup_bindings::tests` module. The successful ordinary
Rust cohort independently records those same six identities. Selecting `--lib`
keeps all six tests, their assertions, the package root and its default features.
The ordinary required Rust cohorts and full merge-queue archive continue to
cover the integration targets.

The two explicit fact targets remain unchanged: six `vize_l0::fact_demand`
tests and seven `vize_croquis::fact_spec` tests. All three commands now use the
existing `ci` profile, as the subsequent real CLI build already did. Both the
default test profile and `ci` inherit development settings; `ci` retains debug
assertions and overflow checks while disabling debug information and
incremental output. No package selection or feature flag changes.

Rust is selected through `rust-toolchain.toml`, using the existing repository
installation pattern instead of another version literal. The target namespace
is `nextest-ci`, which the existing main-push and daily seed workflow prepares.
Cargo still validates its own fingerprints and feature combinations. The
existing cache action's PR read-only policy and trusted writer are unchanged.

The job name, triggers, hydrated three-project corpus, full mutation recall,
diagnostic identity checks and CLI build are retained. In particular, all
90 matrix and 130 corpus-shard mutations still have to be detected with no
misses, baseline mapping failures or unexpected diagnostics.

## Acceptance

The [first repaired run](https://github.com/ubugeeei-prod/vize/actions/runs/38889361409)
on `99a36285bc978bd30c66ca1cf3adcf5a3c3f2b29` passed in 160 seconds, down from
491 seconds (67.4%). The independent specs took 56 seconds, including 34.89
seconds of Patina compilation; the CLI step took 54 seconds. All six L0,
seven Croquis and six Patina tests passed, together with the unchanged 90
matrix and 130 corpus mutation checks. The run used Rust 1.99.0 and mounted
the prepared target read-only; it did not write a PR cache.

This improvement does not establish the five-minute whole-PR target. The next
ordinary run on `9f435f0` passed every workflow but took 7m03s overall, with
slower native builds in the JS jobs. PR and full merge-queue Rust test
inventories are compared at the same source head separately.

Evidence: [initial job](https://github.com/ubugeeei-prod/vize/actions/runs/38882495106/job/116912317210),
[updated job](https://github.com/ubugeeei-prod/vize/actions/runs/38884755975/job/116919687726),
[passing required Check](https://github.com/ubugeeei-prod/vize/actions/runs/38884756035),
[Cargo profile settings](https://doc.rust-lang.org/cargo/reference/profiles.html).
