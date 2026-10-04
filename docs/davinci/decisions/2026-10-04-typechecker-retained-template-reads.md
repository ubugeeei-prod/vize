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
file and diagnostic ordering; strengthen the independent harness to retain
that ordering, compare every planted pair and archive the actual CLI's complete
virtual files outside timings before treating output-parity evidence as final.

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
