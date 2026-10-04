# Original formatter instruction measurements (#6882)

The qualified history retains the original 56 fixes and 300 API plans. Its four
Criterion smoke calls and the existing 100-probe instruction gate contain no
formatter-specific measurement. Measure the actual four original public
formatter routines with the existing benchmark-only StageWindow/Callgrind
protocol. No production formatter route, native equivalence or default switch
is introduced. Genuine OXC printer-error execution remains unresolved.
The paired decision is recorded on [#6882](https://github.com/ubugeeei-prod/vize/issues/6882#issuecomment-5974773462) and [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-5974773581).

## Original source and input custody

Start from signed actual main `d013629e9ebc190bdd6e1f2780d54b20d4bee4a0`.
Keep `crates/vize_glyph/benches/formatter.rs` byte-identical, including historical
source S021: 5,222 bytes, SHA256
`f84876728c734572b018dec9da79917ceadd46e57d9ab7b48ae515ddebcdbea5`.
Its three literal bodies are copied exactly into separate immutable input
fixtures: SIMPLE_SFC 454 bytes, LARGE_SCRIPT 1,083 bytes and COMPLEX_TEMPLATE
1,279 bytes. Leading and trailing LF bytes remain original. These are input
references, never generated formatter output or instruction expectations.

The new benchmark-only target calls the same four public routines with the
same complete expressions, black_box arguments, unwrap and full return value:

| Probe                      | Original public routine   | Input            |
| -------------------------- | ------------------------- | ---------------- |
| formatter_sfc_simple       | format_sfc                | SIMPLE_SFC       |
| formatter_sfc_reuse        | format_sfc_with_allocator | SIMPLE_SFC       |
| formatter_script_large     | format_script             | LARGE_SCRIPT     |
| formatter_template_complex | format_template           | COMPLEX_TEMPLATE |

Each routine gets original FormatOptions::default() outside the measured
window. Reuse has exactly one captured Allocator::with_capacity(8192), with no
reset or prewarm. Ordinary SFC/script API-owned allocation stays inside its
public call. The complete call and unwrap occupy exactly one StageWindow;
setup and returned-value destruction remain outside, using the existing
stage-return protocol. This is not a byte-identical Criterion sampling/timing
window: original Criterion warmup and iteration/drop behavior remain intact in
its separate unchanged target.

## Measurement and admission

Preserve every existing 100 registry row, numeric ceiling, fixture/window
identity, methodology and immutable ratchet. Benchmark harness access is a
dev-only dependency of the legacy formatter crate. No new level dependency,
production pass, pipeline stage, serialization or product warmup is added.

New ceilings require three genuine source-qualified hosted executions with
identical positive counts and complete input/window/binary/raw process receipts.
Use the pinned Linux x86_64 Rust, Callgrind, allocator, guest context and libc
protocol. Existing Actions supplies collection; local Rust builds, installs and
manual duplicate full campaigns are not used. Initial calibration is bounded
and explicitly unbudgeted; it gives no protected admission or guessed ceiling.
Only actual authenticated identical counts may become the exact new ceilings,
with no headroom and no raised old cap. Final source Actions and independently
qualified protected execution/actual merge remain required after freezing.

The collector's explicit --formatter mode selects only vize_glyph's new
formatter_instructions target. Its separate four-key metadata registry contains
no invented allocation or instruction count. The default twelve-suite/100-row
collection, fixed guest directory and both old budget files remain unchanged.
The formatter process uses its own fixed guest directory so the two actual
collections cannot reuse each other's executable links. The existing hosted
instruction workflow retains all old measurement/enforcement steps and adds a
separate three-run collection plus always-uploaded full raw packet. The cap
freeze adds unconditional registry/ratchet verification and mandatory formatter
measurement enforcement through the same required aggregate. First introduction
requires the exact reviewed initial cap-file checksum; later immutable bases
must preserve every row and may only lower its ceiling. Missing new evidence
cannot grant admission. This draft must not enter the protected queue until
fresh final-source acceptance.

## Authenticated calibration and exact cap freeze

Draft source a41c842b was measured by run [37163400887](https://github.com/ubugeeei-prod/vize/actions/runs/37163400887),
job 111321340463, at genuine PR checkout c225f117: accepted main 09b03523 plus
the authored a41 head. The complete CRC-qualified artifact 11287839981 and its
authenticated digest retain actual source, binary, environment, three raw
Callgrind sets and complete input/window identities. Independent review joins
each source fixture to the original literal and each named dump to its receipt,
with zero process-termination leakage and actual allocator preinitialization.
The cap-freeze decision is paired on [#6882](https://github.com/ubugeeei-prod/vize/issues/6882#issuecomment-5974853181) and [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-5974853351).
All four positive counts are identical across three fresh executions:

| Probe                      | Exact initial ceiling |
| -------------------------- | --------------------: |
| formatter_sfc_simple       |                293575 |
| formatter_sfc_reuse        |                274723 |
| formatter_script_large     |                929044 |
| formatter_template_complex |                244347 |

Every window is stage-return. The actual formatter benchmark binary was built
with Cargo fresh=false and SHA256
`cff4ab490e5d9a0bff96d96a3df28a18d8c79dbb398fa0f11d3134d4fab7e240`.
The frozen formatter-instruction-budgets.toml is byte-identical to the actual
measured-budgets.toml, SHA256
`e49d7c38a43fcdfa3bf7114c50ef91734f355d943269d42bc1d18821d444de99`.
No arbitrary allowance or raised existing cap is introduced. The separate old
100-row artifact 11288403585 also passes all three actual executions and the
unchanged ceilings/ratchet in the same job.

Initial source Check 37163401137 retains one genuine formatting failure: the
new template-call closure needs multiline layout. That whitespace-only repair
is included with the cap freeze. The successful calibration does not admit the
repaired final source or protected candidate. Fresh final-source Actions,
protected full/104-row execution and actual merge remain pending. Applicable
allocation/wall/RSS metric coverage, performance improvements, native support
and original printer-error execution are not inferred from these counts.

## Accepted public configuration/CLI prerequisite

PR #7656 actually merged on 2026-10-03 at 23:25:41 UTC as signed `d013629e`.
Source ffbb65ec Check 37160372408 and protected Check 37160846382, Musea
37160845953 and Nuxt 37160845975 are terminal SUCCESS. Candidate artifact
11288065133 retains its own whole 2,902,301-byte public Vite+ report, SHA256
`361c15e6ffa704d6f18ba62573246930824edde1b9f424687922e025ea8f75e6`.
Independent audit qualifies all nine complete public/metadata/native
comparisons, 27 real packaged Node CLI calls, full JSON/argv/streams/files,
source-built addon custody/cleanup and original accessor/callable identities.
The actual fresh=false emitted/generated/frozen addon and each real child load
join SHA256 `92200ac793144254906e8071f0d084ce3802750a544b101f4788c1e3484e957b`.

Instruction job 111313775623/artifact 11286798404 proves the original 100 rows
across three actual executions, twelve suites each, unchanged ceilings and
ratchet. Those suites contain no formatter and grant no formatter metric
credit. Original controls/expected references and failed source packets remain
preserved. Native handled/equivalent/paired remain zero. This accepted public
sorting history does not establish native whole-SFC formatting.

## Genuine original printer-error authority

The original arm is script/format.rs print().map_err(ScriptFormatError), against
pinned OXC formatter/core 0.60.0 at fc702c1fa9f0412d06ec6908b58cd395b826cf7f.
Its actual PrintError::InvalidDocument covers mismatched, missing or expected
IR tags. Standalone script API/CLI propagates the first-pass error; SFC script,
template expression and later stabilization routes preserve fallback instead.
Resolver settings also use ScriptFormatError, so their errors do not qualify
this arm. Synthetic malformed IR, injected failures or unrelated native Doc
errors cannot substitute. No genuine authored-input printer failure or global
unreachability proof has been established; that obligation and #6882 stay open.

## Terminal source and protected acceptance

PR [#7673](https://github.com/ubugeeei-prod/vize/pull/7673) actually merged at
2026-10-04T00:36:00Z as signed valid
`3e67e057348c154aab294c7178510ea6cc5c4d6d`, with a null queue entry.
Final authored source `c079afef` passed Check 37164018765 and actual instruction
run 37164018406. Its independently qualified formatter artifact 11288906238
and original artifact 11288214913 retain all 104 rows across three executions.

[Protected Check 37164617448](https://github.com/ubugeeei-prod/vize/actions/runs/37164617448)
passed all 39 jobs; Musea 37164617160 and Nuxt 37164617167 also passed. Actual
instruction job 111324892287 emitted formatter artifact 11289310675 and original
artifact 11289491015. Independent complete raw-dump/identity/input/window
qualification verifies all 104 rows × three executions, zero termination leakage,
the exact four ceilings above and every unchanged old ceiling. Original registry
and cap bytes equal immutable base `495cdcb9`; the new cap file retains its exact
authenticated first-introduction checksum. The actual Cargo fresh=false binary
has SHA256 `5d5ebddc2d84ec18b799fad1218f715bb712aacf8f64407842e49a88345a1c2f`.
Calibration, final-source and protected receipts remain separate. The terminal
decision is paired on [#6882](https://github.com/ubugeeei-prod/vize/issues/6882#issuecomment-5975058999)
and [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-5975059128).

This supplies actual original formatter instruction acceptance. Historical
failed attempts and at-freeze pending statements above retain their source/time
qualification; no allocation/wall/RSS, blanket original-control, printer-error,
native equivalence or default-path credit is added. The separate
[fixture-gate assessment](./2026-10-04-formatter-fixture-gate-assessment.md)
accounts for the original requirements while preserving the printer TODO on
#6847 and keeping #6882 open until its concrete decision record is accepted.
