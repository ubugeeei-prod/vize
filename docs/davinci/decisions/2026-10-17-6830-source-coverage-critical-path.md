# Full source coverage critical path

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The successful source-coverage job in
[Full Check 38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789/job/117265390323)
at `e2b883ee` took 28m33s including setup and cleanup. Its report step took
27m30s. The complete workflow failed elsewhere after 28m41s.

| Observed phase                        |         Duration |
| ------------------------------------- | ---------------: |
| Cargo build, including linking        |            9m18s |
| Discover 1,219 binaries               |           35.96s |
| Execute 18,532 cases with eight slots |       13m52.468s |
| LLVM profile merge and export         |        3m33.330s |
| Evidence uploads                      | Approximately 3s |

All cases passed; 44 were excluded by the existing configuration. The report
contains 4,249 source files and 474,654/554,696 covered lines (85.570114%).
The sum of case durations is 6,643.707 seconds: approximately 7.981 occupied
test slots, which is not a CPU-utilization measurement. Reordering offers
little idle-time recovery. Existing logs cannot separate compilation/linking
or LLVM merge/export, and do not record peak RSS or raw-profile storage size.
Even removing the entire build leaves approximately 19m15s. Cache work alone
does not establish the requested ten-minute complete release.

After the current correctness failures are repaired, measure these bounded
candidates separately against the same complete source and case inventory:

- Select the existing Cargo `ci` profile while retaining nextest's `full`
  profile, eight slots, source instrumentation and all coverage thresholds.
  Its pinned SHA-256 dependency optimization has separate timing evidence;
  there is no measured coverage speedup yet.
- Opt the `coverage-source` dispatch role into a never-committed clone of the
  trusted main sticky disk. The scheduled main writer was observed in
  [38871318436](https://github.com/ubugeeei-prod/vize/actions/runs/38871318436/job/116875833997).
  The failed Full run had no sticky mount and no Actions target entry.
  Preserve checkout/repository/path checks, disabled target saves and cold
  fallback. cargo-llvm-cov deliberately cleans measured workspace packages,
  so this preserves dependency work rather than the entire measured build.
- Measure raw-profile count/bytes and LLVM merge/export separately before
  considering a bounded online profile pool. Any change must retain complete
  same-source per-file coverage, fresh-profile cleanup and failure detection.

Do not enable `--no-clean`, increase test concurrency or change coverage gates
based on this audit. The existing nextest decision records real timeouts and
resource failures at 32 slots; no controlled larger-concurrency result exists.
Main #9317 requires complete exact-source Full Check before publication, so
this cost belongs to release qualification even when publication is faster.
No coverage implementation or experimental run was changed by this audit.
