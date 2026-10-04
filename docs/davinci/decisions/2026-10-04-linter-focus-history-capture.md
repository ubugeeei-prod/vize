# Original focus-attribute exploratory capture

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881), still OPEN.
Paired [scope decision](https://github.com/ubugeeei-prod/vize/issues/6881#issuecomment-5975616077).
Independent reviews of the frozen source and protocol found no concrete blocker.
This change carries out the authorized exploratory hosted capture on 2026-10-04.
It changes no production rule, default route,
existing shared observer, registered manifest, oracle index or frozen output.
Its first hosted packet remains exploratory and **unaccepted**. A later
complete-output review, adoption decision and paired issue/central record are
separate work; this capture grants no historical or native acceptance.

Created from actual `origin/main` `db31cc119f4a00c48ca67e95c84f4e6790c0c301`, fetched
before creating branch `feat/patina-focus-history-capture` with `wt`. The dirty
earlier audit worktree remains untouched. The new independent example and input
pack do not modify the #7688 observer lane or the existing 44 registered cases.
Frozen private source `1937a6bad23089e38009f3acbb84f462b0841373` passed independent
source/protocol review. Publication replays it onto literal main
`1d91f0aed0ac7496e5f334801f49afb598215332` without changing the producer, original
input pack or protocol bytes; only this qualification and paired records change.

## Original source custody

The exact eight inputs are in
`crates/vize_patina/tests/fixtures/focus-history/cases.json`, SHA-256
`5ab754f0a882fc9c7f8eb843edf454af7909793debe985c0096daa6b132708ba`.
Every row records full fix/parent/witness revision/blob identities, original
rule path/test name, exact source bytes and UTF8 SHA-256. The complete JSON
wire input, input hash and original source hash remain in the capture packet.

| Family                    | Exact fix                                  | Exact parent                               | Witness blobs                                                                                                      |
| ------------------------- | ------------------------------------------ | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| Bound attributes (#1247)  | `c3f6780cb9f7642503137ccf78797b3faf215725` | `c0402a3b3d16c13fa94ff8eeb16b7bb7c2693085` | Autofocus fix `101fccd9272e0174d4f88bf02662f0033e16435d`; accesskey fix `206d5e56818bd4be09f0aa55fce8f5477fd2bdf8` |
| Dynamic accesskey (#2415) | `af1c53419871cfbba89a2e7a8b979dabf0994aa1` | `a12211715ea770ecaa14322844608186a2deca36` | `5cf11b1ffbe282c261f035b9b4bc45ca9ba87158`                                                                         |
| Dynamic autofocus (#2419) | `2a3b4771b244e6bf51fdf4012b2faa9588594cdf` | `af1c53419871cfbba89a2e7a8b979dabf0994aa1` | `33b5723f5439246fd41070f5170346ee3391770b`                                                                         |
| Inherited #1247 controls  | Same bound-attribute fix and parent        | Witness revision is the exact parent       | Autofocus `678b4d696a042f28d94d245a8af3d963847a11e0`; accesskey `17c2cdd179248b8df6baa05ba7a831e999b7efeb`         |

The two files are `crates/vize_patina/src/rules/a11y/no_autofocus.rs` and
`no_access_key.rs`. The original helper creates `RuleRegistry::new`, registers
exactly one concrete `NoAutofocus` or `NoAccessKey`, then calls
`Linter::with_registry(registry)`. Every original query is bare
`lint_template(source, "test.vue")`. There is no authored SFC wrapper, preset,
enabled/disabled override, Vue/Vapor request, locale/help setter or severity
override. The new example reproduces that concrete constructor, rather than
registering a rule name or using an Incremental/default preset.

| Input ID            | Original test                           | Original warning-count authority |
| ------------------- | --------------------------------------- | -------------------------------: |
| `autofocus-bind`    | `test_invalid_has_bound_autofocus`      |                                1 |
| `accesskey-bind`    | `test_invalid_has_bound_accesskey`      |                                1 |
| `accesskey-dynamic` | `test_valid_dynamic_accesskey_argument` |                                0 |
| `autofocus-dynamic` | `test_valid_dynamic_autofocus_argument` |                                0 |
| `autofocus-absent`  | `test_valid_no_autofocus`               |                                0 |
| `autofocus-static`  | `test_invalid_has_autofocus`            |                                1 |
| `accesskey-absent`  | `test_valid_no_accesskey`               |                                0 |
| `accesskey-static`  | `test_invalid_has_accesskey`            |                                1 |

These count-only original assertions remain unchanged. They cannot establish
complete diagnostic/error/help/label/fix output or prove an empty result.
The pack records the counts as witness metadata and preserves any unexpected
actual diagnostics or offered edits for review; it does not construct goldens
from these counts or inferred source spans.

Raw `vue_version`, `vapor`, `locale`, `help_level` and `severity` stay explicit
JSON `null` and Rust `None`. Current constructor source sets
`Locale::default()`, `HelpLevel::default()`, no severity overrides and no
requested Vue/Vapor values. `RuleIdentity` records the actual concrete Rust
type, metadata name/description/category/fixable/default severity, actual
single registry names and `linter.locale()`. Its
`constructor_default_help` records the public default used by this current
constructor; it is not a claim that historical authors selected `En`/`Full`.
The captured actual diagnostic help also retains the constructor's effect.
Other locales/help/severity configurations require later independent complete
observations; they are not covered by these eight original option witnesses.

## Complete producer protocol

New example: `crates/vize_patina/examples/focus_history_observer/` with normal
`main.rs`, `case.rs` and `observe.rs` modules, each under 350 lines.

`--contract` returns exact newline-terminated JSON schema
`vize.focus-history-observer`, version 1. It declares only `--legacy` and
`--native`, the original entry/filename/one-concrete-rule constructor,
`original-unspecified` options, `Case+RuleIdentity+Observation` output,
`capture-only` acceptance and `fallback: false`. The adapter validates the
exact bytes/hash and semantic content; a successful probe alone is insufficient.

Both capture APIs reject unknown input fields. Rust optional fields accept
missing keys as `None`; general CLI deserialization therefore does not prove
explicit nullable-key completeness. For these eight original wire inputs, the
immutable full pack hash and adapter's explicit-null checks establish that
complete authored shape. Altered or missing-key inputs cannot enter this packet.
Each executable invocation independently constructs and captures the entire
outcome twice, requiring exact whole-byte equality before emitting stdout.
Legacy output is complete pretty Debug `Case`, then actual `RuleIdentity`,
then `Observation`, each newline terminated. `Observation` contains the full
unsorted/unfiltered initial `LintResult`, all actual offered fixes independently
applied to original source bytes with diagnostic index and full requery result,
and a genuine unchanged requery when there are no offers. The diagnostic Vec,
messages/help, spans, labels, offered edits, filename and both counts survive.

Native queries use the same original concrete configured linter and actual
public `lint_native_template` for initial/application/requery. The two concrete
instances currently lack `Rule::as_native_template_rule`; the real driver
therefore returns `NativeTemplateLintRefusal::UnprovidedRule` before source
parsing on every input, including absent/static/dynamic controls. The producer
serializes that actual enum's kind/full Debug detail/requested rule and complete
Case/identity context. It does not branch on fixture IDs, fabricate clean
findings, substitute selected SFC APIs or invoke a legacy fallback.

Native JSON has either exact fields `state/context/refusal` for a refusal or
`state/observation` for a genuine future handled result. A refusal has exact
`kind/detail/unprovided_rule` fields. For this fixed unprovided pack the adapter
requires real `UnprovidedRule` with the exact original rule. A future handled
result is retained as a raw failed-scope capture requiring new review, not
silently adopted or credited as a golden.

## Source-built packet and failure preservation

`tests/differential/focus-history{,-capture,-report}.ts` reuse the existing
source-build receipt helper with a new distinct example identity. A real hosted
build uses locked Cargo, the explicit `ci` profile, this workspace package,
the exact example source path and no optional features. The generic receipt
binds committed source revision/full tree/product source tree, Cargo.lock,
example main source hash, selected Cargo JSON artifact/target/features/profile,
executable bytes, compiler/Cargo versions, raw build logs and fresh contract
probe. All original source and new helper/fixture inputs must be committed.
The full source tree also binds sibling Rust modules; the main-file hash alone
is not sufficient evidence.

The adapter invokes both APIs twice in fresh processes for every exact input.
It collects all attempts before classification. Per-invocation throws retain
the actual exception, null unavailable status/signal and empty unavailable
streams, and do not skip the second invocation. Process failures, signals,
timeouts, nonempty stderr, malformed output or repeat drift retain both raw
stdout/stderr byte encodings/hashes, actual status/signal/processError and
failure explanation. Compile failures retain the real Cargo logs and a separate
failure marker; no successful receipt or observation is invented.

The packet schema is `vize.focus-history.capture`, version 1, with exact
top-level `schema/version/acceptance/sourceRevision/fixtureSha256/buildReceipt/
rows/summary` fields and `acceptance: "unreviewed"`. It deliberately is not the
registered differential-result schema. Row fields are exactly
`id/inputSha256/inputBase64/sourceSha256/comparison/legacy/native`. Each lane
has `state/argv/attempts`, plus `error` only when failed. An attempt has exactly
`stdoutBase64/stdoutSha256/stderrBase64/stderrSha256/exitStatus/signal/processError`.
Unknown accepted/provenance/reference-comparison fields are rejected.
Status is a nonnegative integer or null; signal is a `SIG...` string or null;
processError is a nonempty string or null, including on failed lanes.
All raw stream encodings must be canonical base64 and hash their actual bytes.

Legacy lane status is `captured` or `failed`; native is `refused` or `failed`.
Every lane retains exactly two attempts. Whole-byte repeat/protocol structure
checks are capture quality checks, not semantic historical-golden verification.
Every comparison remains `not-compared/no-reviewed-complete-oracle`.
`acceptedCompleteOracles`, `nativeHandled`, `nativeEquivalent` and
`pairedComparisons` stay zero even for a repeatable complete exploratory packet.
The report is saved as `target/differential/focus-history/unaccepted-capture.json`;
it never writes snapshots, recaptures old outputs or changes shared registration.

## Actions scope and remaining gate

This change adds one explicit `mergeOnlyToolingTests` row for
`tests/tooling/focus-history-capture-execution.test.ts`. The independent pure
input/scope and protocol contract files stay in T0. The source-built exploratory
execution remains in every full T1
suite, with no case skips or instruction-budget change. The pure truth table
checks the new fixture, observer source and companion document inputs: T0
excludes the real execution and full T1 includes it; direct pure-test changes
retain the pure contracts. Frozen source, schema, failure preservation and this
scope passed the required independent and coordinating source review before
publication. Seven focused pure contracts and twelve source gates pass.

The first exact-head hosted packet must be
retained as a source-qualified artifact and reviewed in full, including default
metadata, all diagnostics/edits and repeat evidence. Only a separate reviewed
adoption may freeze authoritative complete outputs or register these cases.
Meaningful native acceptance additionally needs a genuine bare-owner element
provider and original dynamic-argument negative laws; this pack supplies none.

The shared registration remains 44 cases with its existing immutable manifest
and oracle-index hashes. Historical issue counts remain 499 touches / 255 fixes;
the existing inventory pin `b4f25fb6511075aa531be80d645bb0db8cc151e0` remains
503 nonmerge rows / 258 title-fix candidates / 245 other titles / 48 merge
supplements. No title-only credit, denominator reduction, merge-resolution
coverage, #6881 closure or default replacement follows from this capture.

Local validation: all eight exact fix parents/witness blobs/original test bodies,
sources, filenames, concrete helpers, counts and source hashes were verified
against original Git objects without running products. Seven pure contracts pass;
Rustfmt, Oxfmt and whitespace checks pass. Every new source module is below 350
lines. The existing 44-case manifest/oracle-index hashes are unchanged. Rust compilation,
product capture, hosted execution, golden adoption and protected merge proof
remain pending until genuine Actions and literal terminal delivery.
