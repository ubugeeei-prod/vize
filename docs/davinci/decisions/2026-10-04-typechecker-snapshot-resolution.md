# Snapshot import resolution sharing (2026-10-04)

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Prerequisite: [batch source snapshots](./2026-10-04-typechecker-batch-source-snapshot.md).

## Decision

The batch snapshot shares unresolved module parses, but its import resolver
still keys results by the entire importer filename and holds its map mutex
through filesystem probes. Sibling SFCs repeat the same relative import
resolution while unrelated dependency resolutions serialize on that mutex.
The prerequisite's default-max CLI measurement did not demonstrate a gain;
its imported corpus worker augmentation grew despite the one-thread gain.
This follow-up requires new same-run complete CLI evidence before acceptance.

Share snapshot resolution cells by canonical importer directory and exact
specifier. The existing compatibility resolver depends on that directory
for relative paths, `@/` source aliases and nearest package ancestry. It has
no importer mode or condition input. Its existing `.mts`/`.cts` behavior is
preserved; Canon's contextual package route resolver is unchanged. Parentless
paths and a filename literally named `node_modules` use a distinct full-file
cache scope, retaining their previous context and snapshot hit/miss caching.

Each cell captures a positive result or a miss once with `OnceLock`. Release
the map mutex before probing or waiting on a cell, so independent imports
can resolve concurrently. New snapshots refresh missing files and dependency
edits. Overlay and disk extension precedence, JS substitutions and package
exports and the compatibility resolver's returned paths remain unchanged.

The public resolver normalizes the supplied importer on every call, retaining
current-directory and symlink freshness for saved or new unsaved roots. This
includes every type-world edge, exactly as before. There is no new canonical
path cache or trusted-path shortcut. Root text and normal/setup scope
still come from the current caller. Cycle handling and the existing per-world
512-module admission check remain unchanged.

## Required evidence

- Sibling Vue, `.mts` and `.cts` importers share one resolution while distinct
  package scopes and exact specifiers remain separate.
- Overlay JS/JSX/MJS/CJS substitutions and `@/` alias precedence match existing
  behavior; a cached miss or source revision refreshes in a new snapshot.
- Concurrent siblings publish one cell per import. Public saved and unsaved
  roots observe symlink retargeting and relative cwd changes; cwd mutation runs
  in an isolated test process.
- Symlink workspace package targets match the compatibility resolver's exact
  returned paths. Full exported dependency facts and module identities match
  sibling consumers and fresh snapshots; special file contexts retain cached
  relative/absolute misses until a new revision.
- Existing root-buffer, private binding, normal/setup, TSX, cycle, overlay,
  fresh-batch and warmed admission-limit tests continue to pass.
- The unchanged paired CLI harness builds exact full Stack base/head CLIs and
  retains complete diagnostic and program signatures for generated500 and
  shared-barrel500 in max and one-thread modes. Cold launches, warmed process
  pairs, raw vectors, executable/runtime hashes and separate profiles remain
  required. A successful parity job alone does not reject a timing regression.

Performance acceptance, protected native Stack delivery, real-project and
persistent-session edit evidence, and the full-command 10x goal remain pending.
No benchmark budget or product output is relaxed by this change.

Further canonicalization reuse is deferred: the compatibility resolver can
return its original path when canonicalization fails, so a private trusted
path shortcut requires separate complete path and world-identity proof.

## Corrected prerequisite hold

The leaf prerequisite is corrected at `e027dacb36da6ae5614288fc8fcd7063753b5b16`.
Its [guard receipt](./2026-10-04-check-shared-script-leaves.md) records actual
failures involving template package imports, nested Vue package/alias context,
trivia-separated global declarations, optional JS require and malformed
import/export recovery. Twenty-one local complete ordered cases pass in both
one/two-server modes: 84 base/head comparisons plus 21 separate profiles.
Resolver source/tests are unchanged from `91a4b93b`; the combined graph at
`f7fdb531a1a9811d9d36ee423afda7472d80e1fc` requires renewed exact hosted
contextual/API/CLI parity, all 104 ceilings and same-run full-command timings.
The three timed corpora, generators, protocol and workflow remain identical.
Earlier green checks and timings qualify their recorded historical graph only.
Native Stack queue admission also remains held during the release freeze.

## Historical exact source receipts

[Run 37173344543](https://github.com/ubugeeei-prod/vize/actions/runs/37173344543)
passed all contextual tests, complete module-fact parity and full CLI gates at
source `3712c7cb784848655050221538a6b9cfa8bb0dd9`. The public API probe compares
the immediate leaf prerequisite `5099e67572d50fd4ccae4951bee88d062d9cfb5d` with
that source. All twelve fact signatures and host/module counts match.
Warm 128-host medians show the resolver's isolated effect:

| Workload     | Threads | Base (ms) | Head (ms) |
| ------------ | ------: | --------: | --------: |
| Fanout       |       1 |    77.539 |    51.185 |
| Fanout       |       4 |    45.190 |    23.932 |
| Deep barrels |       1 |    41.559 |    42.431 |
| Deep barrels |       4 |    23.184 |    22.420 |
| Vue/TSX      |       1 |   131.103 |    49.600 |
| Vue/TSX      |       4 |   104.508 |    23.422 |

These stage timings do not establish the whole-command target. Local-only
warm medians are 0.740→0.748 ms in one thread and 0.425→0.393 ms in four.
Dependency cold one-host changes range from -0.146 to +0.027 ms.

The actual CLI compares the full Stack with exact main source
`da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5`. It uses Blacksmith's 32-vCPU
Linux x64 AMD EPYC runner, Node 24.14.0, Rust 1.99.0, release/thin-LTO/16-codegen
units with stripped symbols, Vue 3.6.0-beta.10 and native TypeScript 7.0.2.
Two warmups precede nine alternating fresh-process pairs. Max retains automatic
server/Rayon width; one thread is one server plus Rayon=1, with native runtime
scheduler width left at its default. Profiles run separately after all timings.

| Corpus / mode       | Base median (ms) | Head median (ms) | Paired mean head minus base (ms) | Conditional 95% interval (ms) |
| ------------------- | ---------------: | ---------------: | -------------------------------: | ----------------------------: |
| Shared barrel / max |          347.513 |          327.446 |                          -21.007 |            [-24.988, -17.025] |
| Shared barrel / 1T  |          805.159 |          445.710 |                         -365.872 |          [-377.828, -353.916] |
| Shared leaf / max   |          468.876 |          277.673 |                         -191.515 |          [-199.209, -183.822] |
| Shared leaf / 1T    |          589.875 |          577.878 |                           -8.408 |              [-17.214, 0.398] |
| Generated500 / max  |          433.198 |          432.831 |                           -1.547 |              [-10.509, 7.416] |
| Generated500 / 1T   |         1332.481 |         1351.708 |                            7.700 |             [-13.021, 28.420] |

Intervals use the nine paired deltas and t=2.306 for eight degrees of freedom;
they are conditional on this run and runner. Imported max and 1T, and leaf max,
are faster in every pair. Imported max improves 5.77% and reverses the
child-only max regression. Generated500 max and 1T, and leaf 1T, remain
unresolved; no baseline-corpus or 10x improvement is claimed.

| Corpus / mode       | First base process (ms) | First head process (ms) |
| ------------------- | ----------------------: | ----------------------: |
| Shared barrel / max |                 390.006 |                 331.818 |
| Shared barrel / 1T  |                 840.654 |                 444.877 |
| Shared leaf / max   |                 469.951 |                 285.152 |
| Shared leaf / 1T    |                 582.123 |                 583.324 |
| Generated500 / max  |                 432.949 |                 434.409 |
| Generated500 / 1T   |                1338.673 |                1348.642 |

Cold means the first check process in that row; filesystem caches are not
evicted. All 266 process records retain authored report ordering, complete
diagnostics and programs. Shared barrel has 535 reported files and no errors;
shared leaf has 502 files and its one planted TS2322; generated500 has 500
files and no diagnostics. Both sides/modes pass every minimal and full-corpus
plant, with complete base/head planted-report signatures. The modes retain
the same corpus signatures.

The measured executable SHA-256 hashes are
`1eee7a0fb0c229d0c1cf7e2c7b558e74daf0bb2cc97e8960f7e3e03b73a6e091` (base),
`a79284fab5ec420828a111743633ab434b1700fa2b0850e8b4449b84bd1c4e66` (head), and
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`
(native TypeScript). Both Cargo lock hashes are
`88227fb6c49020dd2299778023e9d86870e3bfd1a61db7575bf373a5d1fa7b5b`;
the head pnpm lock hash is
`9ff62041a5a606e2b3d4a53660bca3ca42c609026b104fd35d960f0f057b127f`.
The artifact archives every script hash, normalized report, profile and input.

| Corpus       | Input bytes including config/package | Input SHA-256                                                      |
| ------------ | -----------------------------------: | ------------------------------------------------------------------ |
| Barrel500    |                               291932 | `1f203112735e8468965f1e02a5bba5fab4c03b9f6b62dd0de95398df7346bb20` |
| Leaf501      |                              2015701 | `fe18976393a810327d43610e50c92f46974c9a5641110003a3c5ee06783190a5` |
| Generated500 |                              2050821 | `5154a517b8fb59d9ffd1c14d5806fbb7fc870594e67124a9250c3b518bebaff4` |

The generated Vue files themselves retain the historical 2,050,350 bytes.
A separate imported-max profile observes summed worker augmentation
1251.252→551.709 ms for 500 roots while the single backend command remains
218.376→221.619 ms. These are one profiled observation and summed worker time,
not full-command medians. Generated max's sharded backend remains
327.612→318.610 ms in separate profiles. The head profile
observes 327.95 ms total backend wall; its unprofiled whole-command median is
432.83 ms. These populations cannot be subtracted to decompose that median.

The next measured lane begins with the pinned native backend's
`--extendedDiagnostics` phase receipts before attributing startup, program
construction or checking costs. An isolated per-shard incremental-state trial
must keep build-info outside virtual-tree pruning, compare full virtual bytes,
mapper state, program membership and ordered diagnostics, and prove config,
package and edit freshness against a fresh checker. Fresh CLI, disk-state-warm
processes and live sessions remain separate populations. Persistent no-op
experiments must assert session starts/reuses/fallbacks: the current
`check_incremental(&[])` path invokes a fresh CLI rather than a session no-op.
No backend cache speedup has been measured or accepted. Real-project edits,
source freshness and the original fresh-command 42.55 ms/10x target remain
unfinished; native product and fix-history gates still apply.

Receipt `91a4b93b9d3d8b9946998da22f9aaadea97d5b4c` changed documentation only
relative to measured source `3712c7cb784848655050221538a6b9cfa8bb0dd9`.
The corrected prerequisite changes the combined source graph, so those old
receipts do not authorize acceptance or queue admission. Renewed exact-head
Actions and actual protected Stack delivery remain required.

## Raw full-command pairs

Samples are unprofiled wall milliseconds in original pair order. Raw process
and complete API fact signatures remain in the linked Actions artifacts.

```json
{"row":"shared-barrel500-max","baseMs":[343.3749689999995,344.1619499999997,343.8140549999998,347.512522,350.1071869999996,353.248012,346.1052609999988,352.9640280000003,351.7213269999993],"headMs":[327.4459790000001,326.08227199999965,319.93508499999916,330.5816649999997,331.2336029999997,332.83895500000017,326.1906650000001,319.76782999999887,329.8734919999988]}
{"row":"shared-barrel500-1t","baseMs":[804.938704000002,811.0931029999992,803.9018890000007,805.1592569999993,799.6877540000023,804.4246450000028,829.2773290000005,820.6201849999998,812.7452850000009],"headMs":[442.73194800000056,440.3205259999995,456.9231140000011,450.48521999999866,447.1858250000005,446.014610000002,440.31149000000005,429.31888700000127,445.70975599999656]}
{"row":"shared-leaf501-default-max","baseMs":[465.20264999999927,480.6463980000008,468.8763859999999,472.3399169999975,485.1880289999972,471.734838999997,461.60621900000115,463.49835200000234,465.2258329999968],"headMs":[274.11096800000087,277.6727749999991,275.76268900000287,277.0037050000028,281.53136999999697,272.9279200000019,282.9410829999979,286.79658299999574,281.93489899999986]}
{"row":"shared-leaf501-default-1t","baseMs":[592.8643789999987,590.3234349999984,583.6359030000021,569.6172389999992,590.9496629999994,589.8747490000023,595.0180820000023,583.5131519999995,581.1002789999984],"headMs":[574.7153869999966,588.4151209999982,578.9808790000025,577.8779930000019,578.8572859999986,574.2912399999987,567.6999090000027,573.6903579999998,586.700503]}
{"row":"generated500-max","baseMs":[433.65429199999926,433.1975319999983,429.54128800000035,454.0268290000022,445.40385300000344,431.56515899999795,429.3648159999939,433.486155999999,433.06014299999515],"headMs":[432.83141199999955,441.1252480000039,442.8875059999991,430.37179299999843,432.0382989999998,434.87258600000496,437.2813910000041,432.28468299999804,425.6885040000052]}
{"row":"generated500-1t","baseMs":[1352.9734199999948,1366.0037520000042,1358.8720250000042,1332.4814279999991,1341.994789999997,1324.7500370000052,1328.0549680000113,1323.803660000005,1328.1398380000028],"headMs":[1321.289076000001,1352.3365410000115,1335.463603000011,1340.6010950000054,1351.7081569999864,1366.6505109999998,1354.526475999999,1367.7167689999915,1336.080283000003]}
```
