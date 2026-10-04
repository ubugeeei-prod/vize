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

Measured gains below are scoped to two synthetic imported corpora. Protected
Stack delivery, real-project and persistent-session edit evidence, and the
full-command 10x goal remain pending.
No benchmark budget or product output is relaxed by this change.

Further canonicalization reuse is deferred: the compatibility resolver can
return its original path when canonicalization fails, so a private trusted
path shortcut requires separate complete path and world-identity proof.

## Admission domain and final union

Leaf `59067a34ce54cb5e99cace756ee27a18f0fd452d` retains production `944da15b00`.
Its tuple-typing repair preserves all 239 inputs and replays 31 parent cases,
124 ordered vectors and 31 profiles; this count proves those cases only.
Admission requires authored source for ALL roots before shared-root exemption.
Ordinary TS/JS and Vue script bodies contain no `<`; Vue requires canonical
TS/JS scripts and HTML templates, without external script source. Unsupported
language, JSX, unavailable/generated source and unknown escaped/decoded loads
decline the whole optimization. Known authored ambient `.d.ts` roots retain
the original shared exception. Missing/malformed materialized options and any
JSX setting decline. Refusal retains the complete original plan and owners,
including independent-component sharding. No general soundness is claimed.

Required fifth [PR #7754](https://github.com/ubugeeei-prod/vize/pull/7754)
preserves five native-trivia visibility repairs after the resolver rebase.
The genuine 20-report pre-repair runtime receipt is retained below.
Its measured complete source `10995eeb3c3d7a54d76b3a8dfab3d9edb074a89f`
passes the 36-case union below; parent-only 31-case results cannot qualify it.
Resolver source/tests remain identical to `401295c324`; timed corpora,
generators, protocol, build modes and all 104 budgets remain unchanged.
Prior green checks and timings qualify only their recorded historical graphs.
Protected Stack delivery and actual merge remain required before completion.

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
Those historical receipts do not authorize the repaired graph. The final
five-layer evidence below qualifies the measured source; any documentation-only
successor must preserve source and pass exact-head Actions before Stack delivery.

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

## Historical successor measurements

These parent graphs retain their exact evidence; none qualifies the repaired five-layer source.
Intervals below are conditional paired 95% t(8), not cross-run confidence.

### Source `91a4b93b`

[Run 37174318908](https://github.com/ubugeeei-prod/vize/actions/runs/37174318908) compares `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` with `91a4b93b9d3d8b9946998da22f9aaadea97d5b4c`; 10 ordered cases pass.

| Corpus / mode              | Base median ms | Head median ms | Paired mean delta ms | Conditional 95% interval ms | Faster pairs |
| -------------------------- | -------------: | -------------: | -------------------: | --------------------------: | -----------: |
| shared-barrel500-max       |        336.573 |        316.171 |              -23.085 |          [-30.355, -15.816] |          9/9 |
| shared-barrel500-1t        |        794.920 |        434.958 |             -358.823 |        [-368.810, -348.835] |          9/9 |
| shared-leaf501-default-max |        457.112 |        269.009 |             -184.959 |        [-190.722, -179.196] |          9/9 |
| shared-leaf501-default-1t  |        566.668 |        566.911 |               -2.511 |           [-12.097, +7.076] |          5/9 |
| generated500-max           |        423.873 |        429.205 |               +1.370 |            [-5.092, +7.833] |          3/9 |
| generated500-1t            |       1322.235 |       1327.586 |               +2.294 |          [-15.122, +19.709] |          4/9 |

Raw paired base/head milliseconds, in capture order:

```text
shared-barrel500-max: base[342.141687,336.573484,337.097338,351.931084,334.131009,335.996361,331.122973,335.678996,349.877890]; head[317.775998,308.534701,308.985116,316.171226,314.497444,326.382484,323.850602,311.455383,319.129499]
shared-barrel500-1t: base[794.753414,794.920167,799.151116,800.921342,780.749556,783.966710,800.123943,803.968768,788.601106]; head[434.958350,439.342237,432.136038,457.304342,439.444489,434.980271,428.260728,423.631621,427.694494]
shared-leaf501-default-max: base[453.467843,458.589897,461.236989,457.111719,454.204798,463.213773,444.898625,459.015185,447.355410]; head[275.660714,262.038782,266.309485,268.280530,276.027683,276.825541,267.460719,272.849081,269.009309]
shared-leaf501-default-1t: base[566.668409,571.465011,575.859506,557.717514,550.104311,563.924794,569.632809,569.583705,565.747409]; head[566.911106,562.056704,565.837177,568.748917,568.682051,551.298574,547.651463,569.560512,567.359427]
generated500-max: base[428.294321,418.228891,422.093161,428.644449,418.452822,435.247635,423.873105,423.723494,426.632618]; head[429.919524,424.176856,420.713459,429.658905,430.443948,418.647958,434.155262,420.602301,429.204510]
generated500-1t: base[1343.769370,1292.193617,1301.530568,1321.937696,1322.235265,1342.743072,1293.342265,1345.030890,1343.910145]; head[1329.493501,1309.143790,1312.640669,1320.364070,1340.566313,1307.658050,1334.615784,1345.267917,1327.586288]
```

### Source `401295c324`

[Run 37176703070](https://github.com/ubugeeei-prod/vize/actions/runs/37176703070) compares `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` with `401295c324635d0bd7205701daabc151453987e1`; 21 ordered cases pass.

| Corpus / mode              | Base median ms | Head median ms | Paired mean delta ms | Conditional 95% interval ms | Faster pairs |
| -------------------------- | -------------: | -------------: | -------------------: | --------------------------: | -----------: |
| shared-barrel500-max       |        348.940 |        331.491 |              -20.700 |          [-24.960, -16.440] |          9/9 |
| shared-barrel500-1t        |        808.640 |        446.331 |             -363.914 |        [-377.481, -350.348] |          9/9 |
| shared-leaf501-default-max |        474.951 |        280.519 |             -199.769 |        [-208.626, -190.912] |          9/9 |
| shared-leaf501-default-1t  |        584.585 |        579.404 |               -5.242 |           [-15.605, +5.121] |          5/9 |
| generated500-max           |        443.722 |        444.490 |               +2.523 |           [-6.579, +11.626] |          4/9 |
| generated500-1t            |       1359.042 |       1365.177 |               +0.079 |          [-23.302, +23.461] |          4/9 |

Raw paired base/head milliseconds, in capture order:

```text
shared-barrel500-max: base[344.499335,364.748341,348.940490,347.675952,356.807280,359.496538,353.609895,347.801108,339.908790]; head[328.790510,338.383099,332.110057,326.723398,331.490778,331.765737,334.860339,324.300945,328.762888]
shared-barrel500-1t: base[827.375259,808.639980,834.935741,804.809860,823.031027,823.768661,793.659049,791.204324,806.809755]; head[446.331114,442.823942,453.703611,453.562816,442.462690,444.545144,439.247361,456.196167,460.130414]
shared-leaf501-default-max: base[485.216842,471.280212,478.758530,474.709847,488.284363,474.950992,471.888020,473.798597,503.512399]; head[278.504431,277.434170,280.518824,281.476050,276.834407,285.469098,276.279961,286.819528,281.145045]
shared-leaf501-default-1t: base[590.147985,616.147763,581.418000,575.309394,584.585091,575.930628,585.843296,573.055659,591.791558]; head[583.723645,579.403856,570.329429,577.453254,583.396867,587.623208,579.229298,573.418383,592.471893]
generated500-max: base[431.881548,457.109718,442.298658,433.622315,446.178896,443.722418,450.082037,438.548969,448.495471]; head[458.380548,450.388377,447.840635,438.562163,442.073694,441.991519,434.167274,444.489847,456.756921]
generated500-1t: base[1354.737673,1350.604529,1369.950852,1398.252296,1338.103562,1359.041550,1346.838356,1381.074708,1375.609077]; head[1345.228496,1406.855351,1375.325311,1369.238046,1340.462145,1353.405518,1365.177448,1395.530316,1323.704801]
```

### Source `0bafe3b9e6`

[Run 37179869039](https://github.com/ubugeeei-prod/vize/actions/runs/37179869039) compares `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` with `0bafe3b9e669b72a7deaba6b17bb3ab82f85d949`; 31 ordered cases pass.

| Corpus / mode              | Base median ms | Head median ms | Paired mean delta ms | Conditional 95% interval ms | Faster pairs |
| -------------------------- | -------------: | -------------: | -------------------: | --------------------------: | -----------: |
| shared-barrel500-max       |        348.012 |        326.420 |              -20.504 |          [-25.081, -15.927] |          9/9 |
| shared-barrel500-1t        |        821.449 |        445.406 |             -371.185 |        [-384.566, -357.804] |          9/9 |
| shared-leaf501-default-max |        467.254 |        278.743 |             -186.481 |        [-193.400, -179.562] |          9/9 |
| shared-leaf501-default-1t  |        588.648 |        578.884 |              -10.398 |           [-24.393, +3.597] |          6/9 |
| generated500-max           |        436.042 |        441.331 |               +4.447 |           [-1.652, +10.546] |          2/9 |
| generated500-1t            |       1343.304 |       1347.104 |               -1.674 |          [-18.506, +15.159] |          5/9 |

Raw paired base/head milliseconds, in capture order:

```text
shared-barrel500-max: base[355.511062,339.560168,344.808365,348.011954,348.168948,346.477275,349.403955,357.090956,343.218842]; head[326.131436,329.359532,320.467580,326.397505,330.414380,328.816483,321.727673,337.981817,326.420058]
shared-barrel500-1t: base[824.210266,795.973403,833.976698,827.507903,828.428099,818.178551,786.662286,804.178557,821.448755]; head[452.528628,446.923964,440.960486,440.377763,445.555907,446.171682,445.405996,441.556215,440.419532]
shared-leaf501-default-max: base[460.614206,468.856880,467.253952,467.956721,457.684310,463.994168,468.560514,474.417191,455.869896]; head[286.448549,281.656230,275.953611,276.436297,279.193240,278.742594,275.393211,272.991605,280.062661]
shared-leaf501-default-1t: base[572.711442,584.308428,603.087037,567.498743,588.764819,597.624198,569.372609,596.344774,588.647730]; head[588.893750,584.684583,570.989031,578.883591,582.265986,559.245080,557.184308,582.962823,569.665980]
generated500-max: base[445.040721,436.041975,438.540264,434.570363,434.808009,435.717757,438.507803,434.392549,438.428463]; head[439.105213,444.813374,448.599880,437.642329,451.481548,435.772219,446.845277,441.331211,430.479547]
generated500-1t: base[1338.014090,1336.629853,1330.361691,1358.127764,1354.374781,1340.875420,1377.767999,1343.303927,1349.040311]; head[1368.306605,1368.608451,1322.297158,1343.392884,1356.243449,1329.974334,1347.103637,1351.156172,1326.351005]
```

## Complete five-layer exact source evidence

[Check 37180536528](https://github.com/ubugeeei-prod/vize/actions/runs/37180536528) and [API/CLI 37180536363](https://github.com/ubugeeei-prod/vize/actions/runs/37180536363) pass at `10995eeb3c3d7a54d76b3a8dfab3d9edb074a89f`.
Policy verification passes for the 100+4 PINNED ceilings and ratchets; actual instruction measurements remain pending in the protected merge queue. Actual CLI baseline is `da66dc241c`; the separate snapshot API proof is parent-relative to `0bafe3b9e6`.
Independent raw audit verifies 396 commands, 144 complete ordered reports for 36 cases, 36 positive native profiles and all plant gates.
Each new trivia case runs one backend command with no sharded span or truncation. The five-layer union preserves all parent controls.

| Corpus / mode              | Base median ms | Head median ms | Paired mean delta ms | Conditional 95% interval ms | Faster pairs |
| -------------------------- | -------------: | -------------: | -------------------: | --------------------------: | -----------: |
| shared-barrel500-max       |        345.461 |        328.316 |              -18.410 |          [-22.329, -14.491] |          9/9 |
| shared-barrel500-1t        |        800.860 |        441.941 |             -358.564 |        [-366.331, -350.798] |          9/9 |
| shared-leaf501-default-max |        464.210 |        281.619 |             -182.902 |        [-191.341, -174.462] |          9/9 |
| shared-leaf501-default-1t  |        578.316 |        572.541 |               -1.930 |          [-14.021, +10.161] |          5/9 |
| generated500-max           |        434.538 |        436.239 |               +3.061 |            [-3.115, +9.238] |          3/9 |
| generated500-1t            |       1356.141 |       1345.065 |               +1.092 |          [-22.580, +24.765] |          5/9 |

The two imported default-width gains reproduce on this corrected source. Generated500 max and both remaining one-thread controls are unresolved; neither 42.55 ms nor 10x is demonstrated.
Intervals are conditional paired t(8); single first-process cold samples do not evict filesystem caches and are not medians. Profiles remain outside timed pairs.

| Corpus / mode              | Cold base ms | Cold head ms |
| -------------------------- | -----------: | -----------: |
| shared-barrel500-max       |      369.878 |      326.280 |
| shared-barrel500-1t        |      821.225 |      442.707 |
| shared-leaf501-default-max |      465.263 |      283.074 |
| shared-leaf501-default-1t  |      582.839 |      572.169 |
| generated500-max           |      443.640 |      433.556 |
| generated500-1t            |     1353.050 |     1352.891 |

Raw unprofiled paired base/head milliseconds, in capture order:

```text
shared-barrel500-max: base[345.460623,350.692608,351.160723,351.238550,339.593504,345.123734,342.509370,341.216244,356.163215]; head[327.090135,334.764391,328.315994,332.056509,333.490972,326.125611,321.431583,320.701892,333.493467]
shared-barrel500-1t: base[799.910729,800.859814,811.495801,804.630385,795.409695,812.688056,799.844143,779.392219,806.186478]; head[447.190796,441.941046,451.353641,441.277602,447.107368,435.041897,437.372015,437.109970,444.942533]
shared-leaf501-default-max: base[472.411188,482.505982,461.031184,459.912468,469.017401,455.674783,464.272996,464.210418,458.477815]; head[279.161001,289.908680,278.011968,281.619091,270.644061,292.677546,280.268145,283.515557,285.594078]
shared-leaf501-default-1t: base[577.877871,578.315963,586.512157,585.184153,573.938517,563.774518,591.390191,566.440364,597.803922]; head[567.788731,602.795610,571.235100,582.293323,587.588636,572.540991,569.270010,570.012075,580.344211]
generated500-max: base[426.959150,435.149052,434.538016,436.137244,433.070759,426.361499,433.892774,442.173018,438.650541]; head[436.239116,443.137524,433.579351,427.954735,436.656977,438.658496,436.163262,432.488266,449.607030]
generated500-1t: base[1314.674184,1356.140866,1360.662827,1376.509432,1356.728518,1368.015714,1346.795126,1351.946495,1337.838543]; head[1343.320899,1372.508508,1332.951812,1345.065051,1328.593292,1353.119167,1362.626382,1344.521032,1396.437385]
```

The runner/build/native runtime and three input hashes match the historical protocol above: Blacksmith 32-vCPU Linux x64 AMD EPYC, Node 24.14.0, Rust 1.99.0, ci-opt thin LTO/16 codegen, Vue 3.6.0-beta.10, native TypeScript 7.0.2, two warmups and nine alternating fresh-process pairs.
Final pinned binary hashes are `fba016bd7269009ae698ca0024a588858279aad706c1a21b2d9d421a22e31633` (base), `d3e38cbc8240f4758fcb7f58e855d9bab19b34a9ebd8173fac70e3b3e16b362d` (head); native and both lock hashes stay as recorded above.
CLI artifact `11294534445` digest is `sha256:ef8e684baa41a1e1ec96b7a02579cfc6c1dab95bdf33b108e1360e520811e53e`; API artifact `11294748918` digest is `sha256:f6f4e646a8f2126d384a87356c3b4eddba762af0be081204932a35edae7ac762`. Both archive exact scripts, inputs and raw receipts.
An independent imported-max profile observes worker augmentation 1123.249→548.334 ms (summed workers), with native backend 228.292→221.439 ms. Generated max retains 317.815 ms sharded / 326.526 ms total backend in one separate profile; these cannot be subtracted from its 436.239 ms unprofiled median.
Native phase receipts and freshness-controlled backend/state experiments remain the next lane above. Fresh CLI, cache-warm processes and persistent sessions stay distinct; no unmeasured cache gain or legacy-backed migration is accepted. Protected Stack instruction measurements, queue entry and actual merge remain tracked separately.

## Genuine pre-repair native-trivia receipt

The [durable old proof](./results/typechecker-shared-leaf-native-trivia-old-probes.json) retains all five source objects, full 20 reports, command records, 20 native-profile hashes and exact source/runtime/scheduler/binary/controller provenance. It is untimed local macOS diagnostic evidence, distinct from the hosted corrected 36-case qualification and earlier static finding.
Source pair: `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` / `f574ba5da9f0660dd8d53dd2c019befd42a8f35c`. Only Comp0 imports loader; all four SFCs import shared.ts requiring package-global LeafGlobal. All five baseline 1/2-server and old-head 1-server six-file/program reports are clean; old-head 2-server adds only `shared.ts error:1:21 [TS2304] Cannot find name 'LeafGlobal'.`
Final case IDs are `reexport-{cr,ls,ps,bom,zwsp}-package-augmentation`: CR U+000D/LS U+2028/PS U+2029 after `// gap`, or BOM U+FEFF/ZWSP U+200B between `export * from` and `"leaf-types"`.
Clean fingerprint is `fb6a6d287551c92821524c9c9c336e2ca699ee13c9359ad6d331bfc9b5c1d0c8`; each old-head2 error fingerprint is `5386de331f523a6e2d996c144da6f0f18aba9dda80db977c58a4b33d6cea75df`.
Original rows digest is `bdd2e2350da59e21ba74c1ab2d9db6f500adff217b891fa98e1842a106c28073`, controller digest `bb3c5b02585158cfd795f6e820a98de6224193920ebe6d0829627ecce82cba8e`, provenance digest `ca6c1b5e1185d2c44500f92643adc598786a43185db160a5a29ff7f228676e7b`.
Archived data SHA-256 is `95a29207cd6f2836dc9af669d8733d1bc492f6afe510b28584166fc90b4fb0fc`; raw capture originated at `/tmp/vize-leaf-native-trivia-old-probe/`. No controller, production, probe or harness code is copied.
