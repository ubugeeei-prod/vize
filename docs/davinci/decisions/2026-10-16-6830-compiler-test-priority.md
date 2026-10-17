# Start the long compiler runtime parity case early

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Give one existing test priority 100 in the nextest `pr` profile:
`vize_atelier_vapor::vapor_custom_directive_parity::custom_directive_payloads_remain_reactive_on_elements_and_components`.
The exact package, binary and test filter changes scheduling only. Keep all
selected tests, package roots, features, concurrency, retries, timeouts, Clippy
and doctests. The `full` profile used by the merge queue remains unchanged.

## Observed idle tail

Both complete Compiler cohorts passed the same 7,154 tests. Their JUnit records
identify the custom-directive parity case as the final test. Times below are
seconds relative to the start of nextest execution, excluding the build.

| Source and Compiler job                                                                                        | Case start | Case duration | Case finish | Latest other finish |
| -------------------------------------------------------------------------------------------------------------- | ---------: | ------------: | ----------: | ------------------: |
| [`99caba0`, job 116919856512](https://github.com/ubugeeei-prod/vize/actions/runs/38884756035/job/116919856512) |     72.214 |        47.619 |     119.833 |              90.117 |
| [`99a3628`, job 116934978218](https://github.com/ubugeeei-prod/vize/actions/runs/38889361559/job/116934978218) |    106.162 |        49.700 |     155.862 |             119.449 |

The corresponding artifacts are
`rust-test-cohort-compiler-38884756035-1` and
`rust-test-cohort-compiler-38889361559-1`. Preserve the complete runtime test;
starting it early lets the short tests fill the remaining worker slots.

## Estimate, not a measured improvement

Replay every recorded test duration on sixteen workers, ordered by its observed
start timestamp. At each start, assign the next test to the earliest available
worker. This reproduces the original makespans within 0.06 seconds. Moving only
the exact custom-directive case to the front gives:

| Source    | Observed nextest wall | Original replay | Priority replay | Simulated reduction |
| --------- | --------------------: | --------------: | --------------: | ------------------: |
| `99caba0` |             119.834 s |       119.816 s |        93.242 s |            26.574 s |
| `99a3628` |             155.862 s |       155.807 s |       122.158 s |            33.649 s |

These are scheduling simulations with fixed durations, not changed-workflow
measurements. Different contention and runner startup can change the result.
The estimate does not establish the five-minute whole-PR target. Build, test
listing, other parallel jobs and sequential report jobs still contribute.

The pinned nextest 0.9.146 supports this setting: priorities from -100 through
100 have been supported since 0.9.91, and higher-priority tests start first.
See the official [test priority documentation](https://nexte.st/docs/configuration/test-priorities/).

## Validation

The pinned nextest 0.9.146 accepted both `pr` and `full` configurations through
`show-config test-groups`, reusing offline Cargo metadata and an explicitly
empty binary list to avoid a build. A temporary priority of 101 failed with the
expected range error, confirming that this check parses the override. This is
configuration validation, not a runtime or test-inventory result.

Comparing the parsed TOML before and after confirmed that removing the one new
PR override restores the entire original configuration. No selection
expression, original override or `full` setting changed.

## Measured Actions result

The first attempt at `986d6cdb14d866bf23786099acbc4f57dff5c824` passed all
7,154 Compiler tests with exactly the same binary-and-test identities as
`9f435f0`: no additions, omissions or duplicates. The priority case started
8 milliseconds after nextest started, rather than near the end of the suite.

| Source and Compiler job                                                                                        | Nextest wall | Case start | Case duration |
| -------------------------------------------------------------------------------------------------------------- | -----------: | ---------: | ------------: |
| [`9f435f0`, job 116936990526](https://github.com/ubugeeei-prod/vize/actions/runs/38889929446/job/116936990526) |    137.955 s |   87.398 s |      50.556 s |
| [`986d6cd`, job 116948362881](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547/job/116948362881) |    110.749 s |    0.008 s |      54.341 s |

The artifacts are `rust-test-cohort-compiler-38889929446-1` and
`rust-test-cohort-compiler-38893209547-1`. These are observed results from
different runs, not a controlled causal comparison. They directly verify the
changed start order and preserved coverage. The observed nextest reduction is
27.206 seconds; changes in Cargo compilation time are separate and are not
attributed to test priority. The fixed-duration estimates above remain
simulations.

All five Rust cohorts, the required reports and all nine ordinary PR workflows
passed. The complete observed PR interval was **8 minutes 8 seconds**, so this
run did not meet the five-minute target. [Check itself](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547)
took 8 minutes 2 seconds, including 144 seconds waiting for the preceding
run's cancellation to finish. From its first job starting to its final report,
Check still took 5 minutes 38 seconds. No extra cohort or duplicated Cargo
build was introduced by the priority change.
