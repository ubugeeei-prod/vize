# Historical formatter fixture-gate raw qualification

This preserves the complete pre-delivery qualification carried by the original
whole-SFC source child. Its then-pending administrative language is historical.
The [current assessment](./2026-10-04-formatter-fixture-gate-assessment.md)
was separately delivered by signed #7703; #6882 closed on 2026-10-04.
These retained details grant no native/equivalence/default/printer credit.

# Original formatter fixture-gate assessment — 2026-10-04

This assesses the original scope of [#6882](https://github.com/ubugeeei-prod/vize/issues/6882): input/expected-output fixtures for each historical formatter fix. The accepted source is signed `3e67e057348c154aab294c7178510ea6cc5c4d6d` (#7673), not a pending branch or a reconstructed native result. Issue closure is held until this record and its paired issue decision are reviewed and delivered with the next genuine source change.

## Immutable requirement coverage

The original denominator is revision `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`, path `crates/vize_glyph`: 87 non-merge area commits, including 56 conventional fixes. The existing [source-bound audit](https://github.com/ubugeeei-prod/vize/blob/3e67e057348c154aab294c7178510ea6cc5c4d6d/tests/_fixtures/differential/formatter-history/fix-history-audit.json) retains all 150 requirements, their minimum distinct arms, original/current witnesses, and immutable case/source pins. None was reclassified or reduced for this assessment.

All 56 fixes and all 150 requirements are accounted for. Every original public-output/error requirement maps to the existing eight API manifests, 300 original plans, and complete input/expected assets. The immutable classification is 54 semantic behavior fixes, one test-reference maintenance fix and one engineering-control fix. The engineering fix `0e2a5bc223bc730ecc70ad71eb064d76e92f781c` (#6455) introduces no new authored public input: it is mapped to checked scanners, moved tests and the actual strict Clippy gate. Its zero-case table entry does not become synthetic public coverage. Source-catalog reconciliation identifies zero missing cases, assets, required arms, witness references or control references. The source-only matrix checks all 600 input/expected hashes, 250 source-catalog entries, 281 historical Git blobs, and 502 witness-catalog entries. Presence and hash conservation are source proof; the actual executions below supply separate runtime proof.

| Fix | Original commit | Requirements | Mapped case IDs | Law/control references |
| --- | --------------- | -----------: | --------------: | ---------------------: |
| 1   | `bcf61ce45afa`  |            3 |               4 |                      0 |
| 2   | `b02f02a6a152`  |            5 |               5 |                      0 |
| 3   | `6b8a1e38f703`  |            1 |               1 |                      1 |
| 4   | `62a134ed0896`  |            5 |               5 |                      0 |
| 5   | `26e85e98df3f`  |            5 |              11 |                      3 |
| 6   | `0e2a5bc223bc`  |            1 |               0 |                      1 |
| 7   | `2f0217ef47c3`  |            3 |               4 |                      1 |
| 8   | `4f07324ccceb`  |            2 |               2 |                      0 |
| 9   | `aff98628b1dd`  |            6 |              19 |                      1 |
| 10  | `b1ef6b73467d`  |            2 |               4 |                      0 |
| 11  | `8f5eddf29ee9`  |            1 |               1 |                      0 |
| 12  | `1ab272292ebe`  |            2 |              11 |                      1 |
| 13  | `4cdb0510f486`  |            1 |               2 |                      0 |
| 14  | `fb462face5f1`  |           13 |              61 |                      3 |
| 15  | `2a6918e63c33`  |            1 |               6 |                      2 |
| 16  | `a9af5e95117a`  |            1 |               9 |                      1 |
| 17  | `f619e2dceb79`  |            4 |               4 |                      0 |
| 18  | `20e7a285d9fd`  |            1 |               2 |                      3 |
| 19  | `8ebaea36738b`  |            5 |               5 |                      5 |
| 20  | `b963c08c70f5`  |            7 |              15 |                     12 |
| 21  | `2379fd4546d9`  |            8 |              20 |                      1 |
| 22  | `58d8538e5e25`  |            3 |               3 |                      0 |
| 23  | `b904a0164884`  |            8 |               8 |                      8 |
| 24  | `97d3ac9f94f1`  |            3 |               3 |                      3 |
| 25  | `4b3cae54feff`  |            7 |               8 |                      7 |
| 26  | `12651dc7367b`  |            3 |               3 |                      0 |
| 27  | `85a067737215`  |            3 |               7 |                      0 |
| 28  | `b184e64ba793`  |            2 |               7 |                      0 |
| 29  | `4496a5184a06`  |            1 |               1 |                      2 |
| 30  | `16c4a71c9759`  |            3 |               6 |                      2 |
| 31  | `47344a8c23e8`  |            3 |               4 |                      0 |
| 32  | `34762d9f3099`  |            1 |               1 |                      0 |
| 33  | `4967ea7c2e59`  |            1 |               1 |                      2 |
| 34  | `19ceb1ad1481`  |            3 |              21 |                      0 |
| 35  | `398bc61f45ed`  |            2 |               4 |                      0 |
| 36  | `c3bec5566ca2`  |            1 |               3 |                      0 |
| 37  | `bed9279f902e`  |            4 |               4 |                      0 |
| 38  | `b76d297f3df0`  |            1 |               4 |                      0 |
| 39  | `d25a270fe1d8`  |            2 |               4 |                      0 |
| 40  | `752416407bc7`  |            2 |               5 |                      0 |
| 41  | `86e9c456c508`  |            2 |               2 |                      0 |
| 42  | `a811a537cc1a`  |            1 |               1 |                      0 |
| 43  | `86ca0ba607db`  |            2 |              12 |                      0 |
| 44  | `59134d67b93e`  |            1 |               1 |                      0 |
| 45  | `a36cf680250f`  |            1 |               1 |                      0 |
| 46  | `822db1ca9dd4`  |            1 |               4 |                      0 |
| 47  | `b9b18337e562`  |            1 |               1 |                      0 |
| 48  | `4abb3d0ec6cd`  |            1 |               1 |                      0 |
| 49  | `8a5632066b32`  |            1 |               3 |                      0 |
| 50  | `8073295273e4`  |            2 |               2 |                      0 |
| 51  | `c06d15fd3d85`  |            1 |               1 |                      0 |
| 52  | `4f198dafe6a0`  |            1 |               1 |                      0 |
| 53  | `5176f1171c83`  |            1 |               1 |                      0 |
| 54  | `b0c5210142d4`  |            2 |               2 |                      0 |
| 55  | `a742ad2f238a`  |            1 |               1 |                      0 |
| 56  | `165dde1b63a2`  |            1 |               1 |                      0 |

The full names, contracts and per-requirement maps remain in the original audit; this table is an index, not a replacement. The 87 count denotes commits. It is not relabeled as an independently verified count of binary references.

The compact [accepted-source receipt](../plan/formatter-fixture-gate-receipt.json) records the exact denominator, current envelopes, protected provenance and remaining boundaries. Full original per-requirement detail stays in the unchanged existing source audit.

## Actual accepted-source observations

[Protected Check 37164617448](https://github.com/ubugeeei-prod/vize/actions/runs/37164617448) completed successfully with all 39 jobs. The candidate literally merged at 2026-10-04T00:36:00Z as signed `3e67e057348c154aab294c7178510ea6cc5c4d6d`, with a null queue entry. Musea and Nuxt checks also succeeded.

| Original contract                      | Actual current evidence                                                                             | Qualification                                                                                                                                                                                                                                                                                     |
| -------------------------------------- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All eight original API manifests       | Artifact 11288877349; 300 unique plans, 852 actual formatter calls and 300 complete options probes  | 276 full-byte successes each have three chained fixed-point passes; 21 typed errors and three internal observations have one pass each. Full arguments, effective options, input chains, output/error bytes, streams, statuses, signals, process errors and summaries match the pinned contracts. |
| Source-built observer authority        | The same artifact retains actual Cargo JSONL and the complete frozen observer executable            | Exact source/lock/observer bytes, Cargo-emitted artifact, frozen binary bytes/hash and reports join to literal accepted `3e67`.                                                                                                                                                                   |
| Five original historical CLI scenarios | Artifact 11289546120; five scenarios in check/dry/write/recheck, 20 actual calls                    | Complete argument/stream/status/file chains prove check/dry immutability, canonical write and successful recheck; standalone source-built CLI receipt is retained.                                                                                                                                |
| Five ordinary shared CLI cases         | The same artifact 11289546120; five cases × three calls, 15 actual calls and ten fixed-point passes | Whole App.vue input/previous-output/expected byte chains, arguments, streams and statuses match. Three Vue-version cases bind explicit configuration hashes; two bind actual `--no-config` defaults. These cases are separate from historical five/20.                                            |
| 118 original Rust law references       | Four protected Rust archive jobs 111326592324/367/377/395                                           | Every reference resolves to exactly one actual PASS; 110 unique functions, no missing or ambiguous matches. Source authority verifies 105 whole-file matches and 13 registered retained-function transitions.                                                                                     |
| Eight preserved-content functions      | The same protected Rust archive jobs                                                                | Every function actually passed. Existing source validators retain whole original/current owners, exact input/options/fixed-point suffixes and complete snapshots. These eight functions are separate from the 118-reference denominator.                                                          |
| Current engineering gate               | Protected Rust job 111324973050                                                                     | Actual strict workspace Clippy executes with `-D warnings -D clippy::wildcard_imports` on Rust 1.98.0 and succeeds. This does not relabel the historical locked Rust 1.95/MSRV campaign.                                                                                                          |

The ordinary shared CLI report does not transport post-execution configuration bytes, a structured changed field or executable bytes; no such credit is inferred. Source-bound standalone receipt and complete actual runtime envelopes remain qualified.

This is one current complete 300-plan batch. Three chained calls on each success do not create two independent full batches. The separately qualified two historical complete runs and the original controlled two-repeat campaign retain their original source qualifications in [the prior capture record](./2026-10-03-formatter-history-qualified-captures.md).

The separately source-qualified `5a56` campaign observed all 24 original engineering/control arms, their source/commit/tree/blob/patch checks, 12 frozen Rust target receipts, process journals, strict Clippy and four benchmark smoke calls. Its owner and independent receipts preserve original `acceptedControls: 0`, `all24ControlsAccepted: false` and pending overall-control/performance/printer semantics. This assessment does not rewrite those receipts into blanket acceptance. Current `3e67` observations do not grant a fresh execution of the diagnostic campaign or separate frozen-binary receipts for archive targets. The original 56-fix fixture scope does not acquire new requirements from later engineering TODOs.

## Supplemental public feature history

The original input/expected denominator is unchanged by later feature tests. Actual protected JS job 111324973008 emitted artifact 11289156737 from this same accepted source. Its original Cargo-emitted cdylib has `fresh: false`; generated/frozen bytes and original child addon loads join to the current source-built receipt. Recorded-data review performs no local native, CLI, formatter, configuration or build execution.

The public NAPI history has nine original plans and 25 actual calls: eight complete output/fixed-point vectors and one complete original public error vector. The packaged Vite+ Node launcher history has nine plans and 27 actual calls against the supported ordinary `vite-plus`/core 0.2.9 peer. Independently authored expectations validate complete resolved public objects, symbol metadata, non-invoked accessor descriptors and original function identity, native configuration, temporary JSON bytes, exact requested Node command/arguments, actual addon loading, streams/statuses, all file effects and cleanup. Every row passes; all original five script controls and inheritance/false/object vectors remain intact. #7656's separately accepted public history and #7258's feature record retain their own historical receipts.

The current supplemental sorting API has 14 plans, 36 actual formatter calls (11 successes × three chained passes and three typed errors), and 14 complete option probes. The configuration CLI has six plans/12 actual calls; malformed configuration has eight plans/16 actual calls. Every complete stream, file/state chain, effective argument vector, status and source-built receipt passes independent raw qualification. These plans remain separate from the original 300; none expands or substitutes an original fix requirement.

## Original formatter instruction acceptance

[The four-routine metric change](./2026-10-04-formatter-instruction-metrics.md) actually merged through #7673. Source-frozen `c079`, its final Actions, and accepted protected `3e67` have separate successful raw qualifications. The four exact initial stage-return ceilings are SFC 293575, allocator reuse 274723, script 929044 and template 244347. Each count is identical in three authenticated executions. The old 100 registry rows and ceilings are byte-identical to the immutable base; every old and new count passes, giving 104 rows × three executions. Mandatory measurement/check/base ratchet remain active. Original benchmark bodies/default options and allocator/window contracts are preserved.

These instruction receipts close the recorded formatter instruction gap. They provide no allocation, wall-clock or RSS measurement claim.

## Remaining boundaries and closure decision

The actual printer error remains an explicit engineering TODO on [#6847](https://github.com/ubugeeei-prod/vize/issues/6847). Original control `oxc-formatter-provider` belongs to non-fix provider/API migration `78d71cbdbb9652612e99330b6e6111902b31d6e8` (#3489), source refs S200–S202. Its actual `.print()` error propagation was introduced without an authored failing input/test. Its linked `script/signature`, `script/zod`, `sfc/signature` and `sfc/zod` inputs are success/fixed-point controls. No original 56-fix/150-requirement row mandates an `InvalidDocument` witness. Current genuine script propagation remains at `crates/vize_glyph/src/script/format.rs:59–62`, while expression printing at `crates/vize_glyph/src/script.rs:162–164` retains its optional refusal path. The original 300 typed errors are one style and 20 JSON errors; later import-sorting setting errors cannot substitute for `.print()` failure.

This establishes the fixture-gate scope and preserves the pending printer arm. No injected printer IR, error alias, native Doc refusal, claimed global unreachability or waiver supplies runtime credit. A genuine source input through the pinned OXC provider is still needed to exercise that arm, or a separately reviewed source-bound provider analysis must establish an appropriate engineering disposition. Neither is claimed here.

The whole-SFC native formatter, complete native equivalence, default path replacement and legacy deletion remain separate product gates. The recorded original differential and public histories keep native handled/equivalent/paired counts at zero. Issue closure must not be represented as native formatter completion or default admission.

The source-bound fixture assessment finds no missing original historical fixture or execution envelope. Final closure requires review and delivery of this assessment and its paired issue decision, while recording the printer TODO under #6847. Until that administrative record is accepted, #6882 remains open.

## Historical paired central wording

The following original child wording is retained verbatim as history; main
already records the separately delivered original fixture scope and closure.

- **Formatter:** L1 only. A rewrite whose safety depends on L2 facts (for example component-dependent self-closing) is a linter autofix instead. [Import sorting](./2026-10-01-formatter-import-sorting.md) records #7258 native/Vite+ controls and byte contracts. [Public Vite+ configuration and npm CLI custody](./2026-10-04-formatter-vite-cli-history.md) adds nine supplemental source-owned plans through the real configuration functions, temporary JSON and unchanged Node launcher. It consumes genuine emitted-addon/live-owner proof, retains whole objects/process/files, the actual local load and noninvoked core-qualified public HMR descriptors with unchanged callable identity; source-scoped `458e517f` hosted capture matches all 9/27 calls, while its warning-red gate requires exact-head repair. Protected full/27-call/all100 acceptance actually merged as signed `d013629e`; native counts remain zero and whole-history/default gates stay open. [Original formatter instruction metrics](./2026-10-04-formatter-instruction-metrics.md) preserves the original benchmark inputs/routines and all 100 existing caps; authenticated a41/c225 calibration freezes four exact stage-return ceilings without headroom; final c079 Actions and actual signed 3e67e057 protected Check 37164617448 accept all 104 rows × three executions with every old cap unchanged. Allocation/wall/RSS and genuine OXC printer-error engineering coverage remain separate unfinished work.

Paired [original fixture assessment](https://github.com/ubugeeei-prod/vize/issues/6882#issuecomment-5975313632)
and [separate printer engineering TODO](https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-5975313832)
are carried with the genuine [whole scriptless source owner](./2026-10-04-native-scriptless-sfc-formatting.md).
The issue remains open until actual delivery and acceptance.

## Historical whole-SFC pairing wording

The following pre-delivery child wording is retained verbatim; its pending
closure state has been superseded by the separately merged #7703 assessment.

Default formatter, script/style Docs and broader Vue/dialects/embeds/options/edits
remain unfinished. The original formatter fixture-gate assessment and proposed
[#6882](https://github.com/ubugeeei-prod/vize/issues/6882) closure are separate
from native product parity/default migration; its issue remains open until the
reviewed assessment is actually delivered and accepted. No native300 equivalence,
legacy fallback/default switch, public FormatOptions parity, oracle or budget
change is claimed.

The same source delivery carries the reviewed [original fixture-gate assessment](./2026-10-04-formatter-fixture-gate-assessment.md)
and its paired issue decision, preserving the complete accepted3e evidence and
historical pending semantics. Its proposed original fixture closure does not
prove native300/public-option parity or switch a default route.
