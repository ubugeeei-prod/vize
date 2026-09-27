# Full merge queue Rust execution

Refs: #6830, #6861. Implementation follows the affected Rust archive/shard
infrastructure in #6918; it does not change product pipeline stages.

## Measured bottleneck

The successful source job [108585919965](https://github.com/ubugeeei-prod/vize/actions/runs/36307142073/job/108585919965)
compiled its full workspace tests in 153 seconds, then spent 1150 seconds in
workspace execution and doctests. The separate main queue job
108589803932 later observed 1108 seconds for that combined phase; its 12781
passed/43 ignored total mixes ordinary tests and doctests and is not a nextest
workspace count. Cargo's completed binary receipts identify
240.60 seconds for 60 mounted-behavior cases, 141.41 seconds for 36 upstream
Vapor cases, 96.24 seconds for 20 Vapor runtime contracts, 89.80 seconds for one
transition case, and 78.28 seconds for 24 patterned-template cases. These are
binary totals from one run, not per-case timing estimates or p50/p90 claims.

## Decision

The merge queue's existing full Rust job remains the sole archive writer and
owns full workspace Clippy, all default-feature doctests, all 11 unchanged
feature-enabled differential commands, and fixture coverage. It archives the
complete workspace once with Rust 1.98.0, the existing Cargo `ci` profile and
nextest 0.9.146. After that job succeeds, the existing four shard workers run
hash partitions of that archive with an unfiltered `full` nextest profile.
There are no new excludes, retries, or ignored tests. Existing declared ignored
tests remain ignored, exactly as in `cargo test --workspace`.

Each full builder and worker requires real TSGO, 100 Nuxt configuration
iterations, and the existing Node/JS/Pkl runtime preparation. The archive
receipt binds source commit/tree, absolute workspace root, platform/architecture,
Rust and nextest versions, Cargo profile, runtime flags and archive SHA-256.
A PR archive with disabled TSGO cannot satisfy a full worker receipt.
Runtime selection exports only the enabled role's flag: full workers never
set `VIZE_TEST_DISABLE_TSGO`, including to an empty string. Existing Rust
helpers treat any presence as an explicit opt-out. Receipts bind real presence
and absence and reject an empty opt-out flag. Workers
restore `target/tmp` and extract to that checked workspace to retain baked paths.
Extension-host guest builds remain serialized within each isolated worker.

The Rust aggregate requires both the full builder and all four shard results;
missing, failed, cancelled or skipped selected jobs fail it. The builder timing
receipt now measures archive construction and doctests separately. Each worker
uploads its actual JUnit and elapsed-time receipt. The builder also retains
the full archive's named JSON test inventory in its existing timing artifact.
Compare ordinary selected and declared-ignored identities against the four
JUnit files; compare doctests and feature-command results separately. The full daily/manual Cargo
recipe remains intact for independent parity checks.

## Validation still required

Focused tests execute the actual workflow shell with controlled Cargo fixtures,
verify failure propagation, reject T0/T1 archive envelope mismatches, and exercise
missing/failed shard aggregation. They do not prove real runtime success.
Before merge, exact-head Actions and the complete manual check must succeed.
The actual merge queue must then execute every selected workspace test, including
the three real-TSGO cases excluded only in T0, all doctests and all 11 differential
commands. Compare named JUnit cases and retain runner/build/shard timings. No
speed budget, whole Davinci acceptance, or fixture-history closure is claimed
until that source proof exists. Overlapping the builder feature tail with shard
execution is a separate possible optimization after this safe graph is proven.
