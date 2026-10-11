# Same-built Rust archive window trial

This manual experiment is a child of the complete artifact transport trial
[#8692](https://github.com/ubugeeei-prod/vize/pull/8692). It changes no existing
producer, qualification, profile, compiler stage, partition or upload default.
Global adoption remains unfinished until full producer/consumer integration and
current physical qualification are proved separately.

The existing manual walltime workflow delegates only its explicit
`rust-archive-window` choice to the reusable experiment, with read-only contents
and Actions permissions. PR events and the `all` choice do not admit this job.
The artifact trial consumer keeps its checkout and delegates the identical
ordered download and source-bound restore steps to `restore-artifact-payload`;
its caller passes the same two identity inputs. Receipt upload stays a separate
job-level `always()` step through `upload-artifact-payload-receipt`, with the
original inner upload also `always()`, preserving failed-checkout and cancelled
step retention independently of the restore composite's success boundary.
This extraction leaves space for all incoming manual choices and jobs within
the shared 350-line budget without suppressing normal YAML formatting.

The frozen original is artifact `11706049605` from run `38151499533`.
Its API source is `1f463e566cee76d093b3f0c41ee30b7bd1314cdb`; its original build
receipt binds physical source `e057b3dd9e8ebed0dee7f4239a34daa2b410f7a0`, tree
`133c282d0c7404fddaf8ce7d3d159ac6cd3d5a86`, Rust `1.99.0`, nextest `0.9.146`
and Cargo profile `ci`. These are distinct identities and remain explicit.
The original ZIP is 4,821,748,890 bytes, with two STORED members: a 607-byte
receipt and `tests.tar.zst` (4,821,747,915 bytes).

The receipt records the original PR runtime: `VIZE_TEST_DISABLE_TSGO=1`, no
`VIZE_TEST_REQUIRE_TSGO`, and `VIZE_NUXT_CONFIG_ITERATIONS=100`. Both arms preserve
that envelope, profile `pr`, `RUST_TEST_THREADS=4`, `CARGO_BUILD_JOBS=12`, source
fixtures and pinned Pkl runtime. This trial provides no full-native or current
whole-gate qualification. It neither introduces nor widens a test exclusion.

Prior prefix inspection found 1,137 test executables and seven other outputs;
that inspection did not authenticate the complete ZIP or tar. Common linked
code is a possible compression opportunity, not measured equality between
distinct executables. No executable is removed or treated as identical.

The new manual workflow first authenticates the full original API identity,
ZIP byte count and SHA-256, then reads both complete members and their ZIP CRCs.
It keeps the complete original ZIP for 14 days before the one-day original
expires. Signed download URLs, credentials and request headers are not stored.

The producer streams the entire decoded tar, including end padding and any
appended members, and records every member path, type, mode, link metadata,
header metadata and regular-file body SHA-256. It re-encodes that identical tar
with `zstd -3 --long=27 -T4 -c`, bounding the window at 128 MiB. It verifies the
complete candidate frame, decoded tar SHA-256 and exact member/body inventory.
The original build receipt remains unchanged; a separate transport receipt
binds both compressed bodies, raw tar, inventories, codec and driver source.
Every producer and staged consumer helper body and mode is bound to its real
Git driver source, with replacement objects disabled.

Eight fresh consumer jobs run baseline/candidate × original hash partitions
`1/4`, `2/4`, `3/4`, `4/4`. Each authenticates the frozen source, toolchain,
runtime envelope, complete archive body and decoded inventory. Pinned nextest
consumes each archive with the original extraction/remap/profile/partition
command, without an increased decoder-memory flag or another Rust archive build.
Each arm also captures the complete unpartitioned PR listing for coverage.
The final job requires all eight source-bound receipts, identical per-partition
listings, exact actual JUnit coverage of each selected case/result vector, disjoint partitions, and their
complete union. Every stdout/stderr stream, listing, JUnit packet, inventory,
decode/list/run resource report and failure result remains an artifact.

The pinned [list command](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/cargo-nextest/src/dispatch/core/list.rs)
uses the same [default ignored-test filter](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/nextest-runner/src/test_filter.rs)
as the run command: ignored cases remain listed with a mismatch reason and do
not become selected executions. The exact case requirement applies to matching
cases. A control keeps an authored ignored case in both complete listings while
accepting its original omission from JUnit; no ignored-test run flag is added.

Small controls cover CRC-colliding distinct bodies, modes and hidden raw paths,
whole-frame truncation/corruption, removed zero trailer bytes, ZIP digest and
member inventory, original receipt drift, candidate/source/driver drift, and
missing, failed or changed partitions. They run normally and with Python `-O`.
Equal omissions, extras or duplicated actual cases in both arms still refuse;
unknown nextest releases, helper body/mode drift and borrowed Git bodies refuse.

Run [38182205691](https://github.com/ubugeeei-prod/vize/actions/runs/38182205691)
at driver `8c20ec1a99623e50709a66c17fab5798f0152044` authenticated the complete
original ZIP and both CRCs, then retained it for 14 days as artifact `11718407805`.
The complete decoded tar has 1,158 members and 26,503,957,504 bytes, SHA-256
`53d50ee07861c57b2a47352fc1d417864f9915946661875e5e505b571a4e733f`.
Both complete member inventories have SHA-256
`77b58ccc09958c878a8309ac8c1fbe4af3c3b009433b5f5b61a4932e59f9be1f`.

| Producer observation              |      Original |   Long window |
| --------------------------------- | ------------: | ------------: |
| Complete zstd bytes               | 4,821,747,915 | 3,542,835,210 |
| Complete tar decode and inventory |      38.765 s |      38.300 s |
| Decode peak RSS                   |    24,628 KiB |   136,192 KiB |

The candidate is 26.52% smaller; encoding takes 22.546 s. These are producer
observations, with the extra diagnostic storage and verification work retained.
All eight worker packets were authenticated by complete API ZIP SHA-256/size,
ZIP CRCs and listing/JUnit body hashes. Pinned nextest decoded both frames.
Partitions 1/2/3 passed 4,556/4,251/4,169 cases in both arms. Partition 4 ran
4,391 cases in each arm and refused the same original component-slot provenance
case: its real Git checkout was `e057`, but it inherited the orchestration
`GITHUB_SHA=8c20`. The aggregate correctly failed; no paired green is claimed.

Each worker also failed runner post processing because its local composite
manifest disappeared during the frozen checkout. Consumer steps are now planned
directly in the reusable workflow, preserving all ordered commands, conditions,
shells and resolved matrix inputs without adding a source overlay. The move has
its own commit. A real Git control reproduces the provenance refusal with an
inherited driver SHA. A finite launcher now requires the actual frozen Git HEAD
and explicit outer driver identity, then execs only the original nextest
`list`/`run` commands with child-only `GITHUB_SHA` equal to that verified HEAD.
The Actions/receipt driver namespace stays unchanged; runtime receipts must
bind the physical test SHA. No original provenance assertion is edited or skipped.
The five Node laws keep their complete registrations within the zero-warning
budget. The corrective record-only run below qualifies the frozen PR-runtime
transport pair; fresh current-source full gates remain separate.

Corrective run [38184043589](https://github.com/ubugeeei-prod/vize/actions/runs/38184043589)
at driver `563594f41d214ca507233e8c6be9fe841d097b4f` finished successfully:
producer, all eight consumers and complete aggregate comparison passed. Each
consumer bound actual Git source and child `GITHUB_SHA` to physical `e057`,
while retaining the separate orchestration driver identity. Original assertions,
fixtures, ignored-case policy, profile and hash partitions stayed unchanged.
The earlier `8c20` provenance and runner-post failures remain recorded above.

Independent reads authenticated the complete API ZIP SHA-256, size and every
member CRC for custody `11718603717`, all eight worker packets and comparison
`11718897532`. Complete listing and JUnit body hashes, exact actual case identities
and successful outcomes, disjoint partitions and their complete unpartitioned
union passed: 4,556 / 4,251 / 4,169 / 4,391 cases, or 17,367 cases per arm.
Every raw stream and original case result remains in the retained worker packets.

The complete original ZIP is retained again as `11718976738`; its wrapper ZIP
SHA differs from the unchanged authenticated inner original ZIP SHA. Candidate
`11719185576` preserves the same decoded tar and all 1,158 complete members.
The candidate again has 3,542,835,210 bytes, 26.52% fewer than the original zstd.
Encoding took 23.860 s; full decode and inventory took 38.500 / 38.191 s, with
peak RSS 24,532 / 136,448 KiB. Original download plus digest took 42.471 s, and
whole preparation took 108.00 s with peak RSS 163,328 KiB. These retained
diagnostic costs and eight consumers provide one paired observation, not a
whole-CI speedup or permission to change production uploads.

Current full-source qualification, protected queue and actual merge remain
unfinished. The parent full Check refused the shared Playground setup's missing
`vue-vapor-runtime/package.json`; its independently owned prerequisite
[#8715](https://github.com/ubugeeei-prod/vize/pull/8715) must actually integrate
before fresh source qualification. Historical frozen transport acceptance does
not transfer green status to a changed head or replace current native gates.
Fewer bytes alone will not be reported as an end-to-end CI speedup. Re-encoding,
verification and extra diagnostic uploads have costs; this retained experiment
adds storage while leaving original production evidence intact.
