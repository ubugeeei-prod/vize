# Retained template reads for the existing type checker

Date: 2026-10-04. Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).

## Decision

Reuse the existing per-compile `SimpleExpressionNode.js_ast` in the SFC
import-retention visitor when its `raw` equals the node's current `content`
and the existing `js_module_compatible` gate accepts it. That gate preserves
the older wrapped parser's strict module goal; TypeScript-only, strictness,
sloppy-literal and HTML-comment differences keep the existing parser.

The original visitor, builtin filtering, arrow-binding scope and identifiers
precedence stay unchanged. Static nodes and simple identifiers retain their
existing paths. Slot binding patterns keep their wrapped arrow parser, and
absent or stale ASTs keep the existing expression parser. Nothing is cached
across edits, and no pipeline stage or serialization is added.

Production counters `atelier.template_reads.retained` and
`atelier.template_reads.parses` count admitted walks and actual wrapped parse
attempts. `davinci.expr.parses` still counts original expression nodes; it is
not a repeated-parse counter.

## Verification and performance protocol

Meaningful tests compare the retained and fallback identifier sets, including
stale content, missing ASTs, pre-parsed identifiers, lexical scopes, comments,
invalid syntax, builtin names, decoded entities and Unicode. A single identical
public projection probe is built on the exact baseline and candidate and
compares complete generated code, structured mappings, semantic links and
mapper diagnostics. The full CLI report must also remain identical, including
the program file count and planted diagnostics.

Use the same Blacksmith runner for two warmups and nine alternating baseline /
candidate runs. Measure the historical 500-SFC explicit corpus and the
501-SFC shared-leaf default CLI corpus, with max width and one thread. Profiling
and complete projection comparisons run outside timings. Archive exact SHAs,
binary hashes, all samples, diagnostics, projection receipts and counter totals.
The expected shared-leaf retained count is 40,000; the baseline has no new
counters and remains explicitly uninstrumented at those sites.

The instruction ceilings stay unchanged. Actual paired Actions results, full
PR checks and protected queue acceptance are required before merge. The 10x
whole-command target remains unfinished.

## Initial hosted receipts

Both [run 37172478929](https://github.com/ubugeeei-prod/vize/actions/runs/37172478929)
and the independent same-SHA [repeat 37172909624](https://github.com/ubugeeei-prod/vize/actions/runs/37172909624)
compare baseline `7f7b63122456fd066c86bcab7c281c9c6c9d389a` with candidate
`f6b9c6d47922e39d841dbf9d2bd543954f839845`. Each uses nine alternating pairs;
profiling is excluded from timings. These are conditional results from the
named Blacksmith runner, not a universal performance claim.

| Corpus / mode                    | First paired head/base median | Repeat paired head/base median |
| -------------------------------- | ----------------------------: | -----------------------------: |
| Shared-leaf default / max        |                         0.989 |                          1.011 |
| Shared-leaf default / one thread |                         0.990 |                          0.983 |
| Generated 500 / max              |                         1.006 |                          1.000 |
| Generated 500 / one thread       |                         0.988 |                          0.990 |

The one-thread rows are favorable in both runs; a default-width whole-command
gain did not reproduce. Do not queue the draft as a demonstrated default-width
speed improvement. Record any further acceptance decision on #7698.

Both runs retain all 502 shared-leaf report files with one planted TS2322 and
no warnings, and all 500 generated report files with no diagnostics. Public
mapper generated text, structured mappings, semantic links and diagnostics
match byte-for-byte. The shared-leaf profiles show 40,000 retained walks and
zero wrapped parse attempts; generated profiles show 16,500 and zero. Baseline
counters remain uninstrumented. The initial report comparison canonicalizes
file and diagnostic ordering. The final receipt below strengthens this evidence
with original report ordering, every planted pair and complete actual CLI virtual
files outside timings.

## Initial raw timing samples

Times are unprofiled fresh-process wall milliseconds, retained in original
pair order. Baseline/head samples at the same index form one pair. Complete
raw diagnostics, programs, inputs, profiles and projections are in each linked
Actions artifact; these samples remain in this record after artifact expiry.

Run `37172478929`:

```json
{"row":"shared-leaf501-default-max","baseMs":[451.1414960000002,467.70632099999966,462.53184599999986,456.0971989999998,469.5205649999989,462.51986399999987,475.0296700000017,471.11439299999984,455.36383899999964],"headMs":[459.2189929999995,464.4335339999998,467.1301079999994,449.16584899999907,460.88142100000005,455.09076600000117,466.8499009999996,466.10075099999995,457.6425470000013]}
{"row":"shared-leaf501-default-1t","baseMs":[565.9895099999994,572.6136140000017,576.7678589999996,577.5926809999983,571.727726000001,562.3511340000005,569.372124999998,576.6136079999997,561.1336540000011],"headMs":[582.6567690000011,560.1134649999985,570.7869269999974,547.8193630000023,559.91446,566.2465440000014,562.1459890000006,582.3618940000015,563.657432]}
{"row":"generated500-max","baseMs":[427.87011099999654,438.20459399999527,436.92527800000244,434.32893299999705,437.49468999999954,418.56980200000544,427.3507730000056,431.1370170000009,424.29545399999915],"headMs":[432.0800519999975,426.9327770000018,430.04389000000083,436.6857270000037,433.02311600000394,435.12029099999927,429.7114339999971,443.6861179999978,436.4376700000066]}
{"row":"generated500-1t","baseMs":[1377.943005999994,1343.9153340000048,1331.089694000002,1355.0566270000054,1351.3219499999977,1365.154926000003,1339.811142000006,1313.18439699999,1346.857308999999],"headMs":[1314.3299700000061,1334.0902879999994,1336.054673999999,1336.5599650000004,1335.4141489999965,1321.221206000002,1319.605549999993,1313.300615,1377.5331110000116]}
```

Run `37172909624`:

```json
{"row":"shared-leaf501-default-max","baseMs":[482.79593300000033,464.2047160000002,468.2782669999997,460.2747359999994,450.73590199999853,470.316248000001,465.65968000000066,465.40725999999995,469.6610099999998],"headMs":[474.70020899999963,482.14613999999983,473.62081899999976,470.5646799999995,474.2939320000005,471.78791900000033,462.95799500000066,474.34548899999936,465.3782540000011]}
{"row":"shared-leaf501-default-1t","baseMs":[569.0676390000008,595.9936220000018,584.6312859999998,573.5587120000018,589.389726999998,579.2422340000012,579.8129910000025,567.9118699999999,576.7334069999997],"headMs":[563.5975240000007,582.5984849999986,564.8262040000009,563.6703930000003,565.3120889999991,584.6682960000035,563.1066100000025,574.7374359999994,567.1478349999998]}
{"row":"generated500-max","baseMs":[422.42861899999843,434.36229399999866,435.3809130000009,427.54359000000113,446.51118800000404,441.44671400000516,438.61081399999966,447.56020400000125,441.1785039999959],"headMs":[440.32831499999884,439.6869829999996,429.3003150000004,435.0719650000028,438.35805300000357,441.4025099999999,447.37113500000123,426.38621599999897,428.79946099999506]}
{"row":"generated500-1t","baseMs":[1326.4229479999995,1323.846341000004,1343.537006999999,1343.695799000001,1338.9596500000043,1360.7003580000019,1310.632448999997,1332.7096699999965,1333.8242129999999],"headMs":[1313.563930999997,1307.9382289999994,1362.717478999999,1328.951863000002,1315.503134999999,1331.381760999997,1369.5007970000006,1338.1775609999895,1360.795729999998]}
```

## Final hosted proof and disposition

Retained template-read candidate #7723: final evidence and rejection decision

The strengthened [exact-head Actions run 37173568601](https://github.com/ubugeeei-prod/vize/actions/runs/37173568601) passed on `9b43b5e46471cb5786fdc9e891a79fa057a29619`, against baseline `7f7b63122456fd066c86bcab7c281c9c6c9d389a`. All PR checks for that head are green. Production code and the timing protocol are unchanged from the initial two runs.

The final harness preserves original report file/diagnostic order and checks the complete fingerprint for every planted pair, cold/warm/timed/profile run and untimed generation capture. It retained 152 raw process receipts, including 72 unprofiled timed runs, 8 separate profiles and 8 separate virtual-file captures. Public mapper text, mappings, semantic links and diagnostics remain byte-equal.

Every actual CLI-generated SFC virtual file and shared helper is byte-equal between baseline/head and across both thread modes:

| Corpus              | SFC virtual files + helper |  SFC bytes | Helper bytes | Total bytes | Equal manifest SHA-256                                             |
| ------------------- | -------------------------: | ---------: | -----------: | ----------: | ------------------------------------------------------------------ |
| Shared-leaf default |                    501 + 1 | 11,275,366 |       28,193 |  11,303,559 | `a258d8f5fc28d66993b9ec3f7eed1e89c8edbb8e07ff95a60f75cd44d7ad09f5` |
| Generated 500       |                    500 + 1 |  9,510,700 |       28,193 |   9,538,893 | `c1a134d0553411856791ef7d677ef62951120965e3927733d3b6ff80fa5f986f` |

The shared ordinary `shared.ts` input is also unchanged and included in all 502 reported source files. The shared-leaf case retains exactly one planted TS2322 and no warnings; generated500 retains all 500 files and no diagnostics. Profiles still show 40,000 retained walks / zero wrapped parses for shared-leaf and 16,500 / zero for generated500. Baseline counters remain explicitly uninstrumented.

The final paired head/base medians are 1.003 shared-leaf max, 0.982 shared-leaf one thread, 0.995 generated500 max and 0.991 generated500 one thread. Together with the two earlier runs, these do **not** establish a reproduced default-width whole-command gain. Favorable one-thread medians are conditional evidence and do not complete the 10x target.

**Decision:** close #7723 unqueued and unmerged. Do not ship the retained-AST production change, projection probe or independent harness. Preserve the unique documentation, all raw timing samples and the rejection decision in the accepted resolver PR #7728. The 104 instruction ceilings are unchanged; this rejected candidate never enters the merge queue. This issue remains open for the unfinished default-width and 10x acceptance work.

Complete final inputs, raw reports, profiles, public projections, actual CLI virtual files and manifests are in [artifact 11291804132](https://github.com/ubugeeei-prod/vize/actions/runs/37173568601/artifacts/11291804132). Initial exact-SHA receipts remain linked in the preceding comment and unique decision record.

The artifact digest is `sha256:b10e6712f493f1eeae016551567ab3d356c35e7580d9342abb248541a6b26d5b`.
An independent audit re-read and hashed all 2,006 baseline/head virtual-file
pairs (4,012 archived files), covering 41,684,904 baseline bytes across four
rows; every recorded size, SHA-256 and raw byte pair matched.

PR #7723 was closed unmerged at `2026-10-04T03:25:56Z`; `mergedAt` is null.
The issue decision is [recorded here](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5976148245).

Final run `37173568601` raw unprofiled wall-millisecond samples:

```json
{"row":"shared-leaf501-default-max","baseMs":[455.1084660000006,484.3308109999998,479.82582399999956,470.9932580000004,481.84960599999977,461.06771800000024,450.23432299999877,484.1614410000002,463.8213169999999],"headMs":[465.9817550000007,471.068851,471.0073850000008,472.3866589999998,467.94027299999834,485.164764000001,483.45251199999984,461.5928899999999,475.317622999999]}
{"row":"shared-leaf501-default-1t","baseMs":[576.131179,581.2408609999984,594.1337549999989,576.0886089999985,592.0887570000014,576.0679300000011,589.3729800000001,595.6538510000028,592.7658609999999],"headMs":[561.6855910000013,578.824826,583.5601790000001,570.0273390000002,588.1112979999998,575.0498250000019,575.8444089999975,575.4745239999975,577.1310980000017]}
{"row":"generated500-max","baseMs":[445.42308899999625,438.0815220000004,441.2440299999944,452.25828199999523,430.37082999999984,440.85953500000323,433.3932569999961,439.3694000000032,439.85125799999514],"headMs":[439.0545129999955,444.96548800000164,446.06623799999943,437.0948209999988,438.60051500000554,438.6496660000048,442.0855919999958,433.1537559999997,430.79281400000036]}
{"row":"generated500-1t","baseMs":[1367.661779999995,1360.2470489999978,1321.3501319999996,1313.119316000004,1357.4184569999998,1366.5055040000007,1319.0293570000067,1348.5503409999947,1361.399768000003],"headMs":[1353.2986549999987,1347.992201999994,1343.3678999999975,1340.0862650000054,1311.5086860000083,1352.1419289999903,1373.5234389999969,1364.0405060000048,1343.048820000011]}
```
