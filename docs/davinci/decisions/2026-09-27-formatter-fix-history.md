# Formatter fix-history output fixtures

Tracked in [#6882](https://github.com/ubugeeei-prod/vize/issues/6882), before
replacing the legacy formatter path. The issue's 56 past fix-title commits
remain the minimum audit scope. A complete input/output witness is required;
fixed-point or substring assertions alone do not establish full output parity.

## Decisions

- Retain existing semantic and fixed-point assertions. Add full outputs where
  those assertions leave a gap; reference existing complete witnesses without
  claiming duplicate coverage.
- Use Insta's binary snapshots for new output references. Compare the exact
  returned UTF-8 bytes, including CR/LF and the final newline. No trimming,
  newline normalization, sorting, or production formatter changes are added.
  Git text conversion is disabled for prepared assets and binary references.
  Binary references retain authored trailing whitespace and raw EOF spaces;
  Git's whitespace diagnostics are disabled only for those reference bytes.
- Distinguish public `format_script` and `format_sfc` observations from CLI and
  private helper observations. An internal helper accepting a token sequence
  does not imply that the public CSS parser accepts it.
- Captured outputs are current legacy observations, not inferred historical
  complete goldens. Source, executable, options and input hashes stay explicit.
- Prepared SFC input assets use `.vue.txt` with explicit `kind: Vue` metadata.
  They do not silently enter the global `.vue` census before shared corpus
  registration and its acceptance accounting are implemented.

## First script slice

`crates/vize_glyph/tests/fix_history_script.rs` adds six byte comparisons:

| Public API      | Input                       | Options                   | Existing requirement                  |
| --------------- | --------------------------- | ------------------------- | ------------------------------------- |
| `format_script` | TypeScript return signature | default                   | #2035 fixed point                     |
| `format_script` | Chained Zod regex           | default                   | #2035 fixed point and regex           |
| `format_sfc`    | SFC return signature        | default                   | #1965/#2035 fixed point               |
| `format_sfc`    | SFC chained Zod regex       | default                   | #2035 fixed point and regex           |
| `format_script` | Return signature            | internal single-pass flag | #2035 check output                    |
| `format_sfc`    | SFC return signature        | internal single-pass flag | #1974 check output and change verdict |

The four authored inputs are copied byte-for-byte from the existing regression
tests. Default cases retain three real formatting passes; check cases retain
the intermediate-versus-canonical distinction and actual change verdicts.
The single-pass flag is an internal runtime option, not user configuration.
Its intermediate output is a current legacy API observation. Future native
formatting need not adopt that mechanism or add a stabilization stage; product
compatibility must separately compare the actual CLI check verdict and streams.

The fixture manifest records the input and full-output digests, public API,
options, original witness and product source pin. The capture receipt records
the real Cargo-selected test executable, profile, features, source hash,
toolchain and raw log hashes. Logs and the frozen executable remain at the
receipt's local paths; no Actions artifact or remote publication is claimed.

At base `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`, the product source tree
stayed unchanged. A narrow offline build in an isolated target directory took
62 seconds. All six tests passed, then passed again against stored snapshots
with updates disabled; a frozen executable replay also passed all six.
Actions assertion lint rejected duplicated substring assertions alongside the
complete byte references. Omit only those new duplicates; original semantic
regressions and all golden bytes remain unchanged. Capture hashes continue to
bind the original measured test source; latest Actions validates the edited
test source. Shared corpus registration remains pending.

## CSS slice

`fix_history_style.rs` prepares twelve complete public `format_style` outputs
for top-level comments, comment-like strings/URLs/imports, charset removal,
fractional numbers after Unicode, legacy keyframes and the float-max sentinel.
Every successful output is checked through three real calls. Existing helper
call-count assertions remain separate and receive no new output credit.

The historical helper input containing `--élément.5` is invalid CSS: the public
API returns `StyleFormatError`. Keep that actual typed error in a separate
snapshot rather than changing the authored input or pretending it formatted.
The helper's own numeric-token equality remains required.

Five narrow tests passed capture, updates-disabled replay and frozen executable
replay. Strict Clippy also passed. The receipt binds the actual test source,
executable, toolchain and local logs. These are prepared public observations;
shared registration, Actions and native acceptance remain pending.

## Slot, root-tag and raw-close slice

`fix_history_template_gaps.rs` adds thirteen full public byte references for
static/bound legacy slots, templated template/style/custom roots, incomplete
raw closes and unavoidable inner directive quotes. Preserve actual dynamic
evaluation barriers; helper classification is tested separately for bound
slot/slot-scope names. Three focused priority tests passed.

The public template API removes trailing CR/LF even from raw fragments. The
incomplete-close fixtures retain that existing contract and preserve every
other tail byte; they do not invent a closing delimiter or infer helper output.
Four public tests passed capture, updates-disabled replay, frozen executable
replay and strict Clippy. No production formatter algorithm changed. Keep the
glyph import inventory row in this same slice so each stacked head validates.
Shared registration and Actions remain pending.

## Deleted raw inputs and entity branches

`fix_history_template_older.rs` retains the exact pre/textarea/v-pre inputs
whose partial tests were deleted by a later interpolation fix. Twenty full
public byte references also cover raw comments/EOF, recognized and unknown
directive entities, an escaped backtick through indented SFC formatting, and
wbr/uppercase component names. The `&quot;` input is identical to the preceding
quote fixture; reuse that witness without duplicate fixture credit.

Five tests passed capture, updates-disabled and frozen executable replay, and
strict Clippy. These broaden the public output evidence while keeping existing
helper tests. Shared registration, Actions and full-history acceptance remain
pending.

## SFC layout slice

`fix_history_sfc_layout.rs` copies six original whole-SFC input strings exactly:
literal multiline attributes, multiline comments/pre openings, wrapped
interpolations, trailing text and text between interpolations. Defaults and
three-pass fixed-point checks are unchanged; binary snapshots add the missing
complete first output. Input hashes were checked against the original source.
Six tests passed capture, updates-disabled/frozen replay and strict Clippy.
Shared registration and Actions remain pending.

## Directive layout slice

`fix_history_directive_layout.rs` copies six original inputs and option sets
for multiline/verbatim directive values, v-for collections, blank lines and
leading comments, quote/number policy across SFC blocks and pinned suppression
lines. The complete first output and three real calls retain every original
semantic constraint. Six decoded input hashes matched their original tests.
Six tests passed capture, updates-disabled/frozen replay and strict Clippy.
The already complete two-arm template literal witness is not duplicated.
Shared registration and Actions remain pending.

## Shared public API execution

The `formatter-public-api-v1` observer executes the first six manifest cases
through real public APIs and the shared byte comparator. Four default cases
require three matching outputs and real fixed points. Two internal single-pass
cases remain separately counted legacy observations; they do not prescribe a
native mechanism or count as CLI check-verdict coverage.

The shared tooling execution test builds the narrow observer from the actual
checkout, records Cargo's selected artifact/profile/features, freezes its
executable, probes complete actual options and compares raw stdout/stderr.
SFC changed verdicts are retained separately from formatted bytes. CI uses its
existing `ci` profile and target cache; local capture uses an isolated `dev`
target. Real Cargo execution stays in the explicit T1 tooling inventory; T0
retains the pure contract tests and unknown-file selection remains fail-closed.
T1/nightly/manual Actions retain the frozen executable, raw Cargo logs, receipt
and complete API observations as artifacts, including failure observations.
The source guard also rejects untracked non-test Rust files and Cargo manifests;
NUL-separated Git paths keep that guard valid for non-ASCII filenames.
Missing rows, wrong APIs/options, artifact/source mismatch, output drift,
broken pass chains and invented native credit fail closed.

The initial actual execution and three validator tests passed in 4.47 seconds;
the source-guard and T1 selection follow-up passed all six execution/contract
tests and 26 focused tooling checks. Strict Clippy passed. The original proof
remains at `/tmp/vize-formatter-api-observer-proof-20260927`; the restacked
source-built replay is `/tmp/vize-formatter-api-observer-proof-publish-20260927`.
No Actions success is claimed.
Registration for all other historical cases and actual CLI check observations
remains TODO. Native handled/equivalent/paired comparisons remain zero.

## Remaining work

Fresh Actions also required the two touched glyph consumer/v-on inventories
to reflect the added test paths. Refresh only their three actual new rows;
the central decision record retains the same decisions within its line budget.
Twenty focused tooling checks, sixteen affected public Rust tests and strict
Clippy passed after the assertion/inventory repair. Golden and receipt bytes
remain unchanged; the receipts still describe their original capture source.

Existing regression tests now also retain twenty-four complete first outputs
for opaque Pug/Haml/CRLF/src templates, empty/comment-only scripts, SFC block
attribute quotes and style order, dynamic attribute barriers and wrapped
interpolations. Their semantic and fixed-point assertions remain in place.
Seventeen focused tests passed updates-disabled and frozen executable replay;
strict Clippy passed. The aggregate receipt binds all five actual executables
and test-source hashes. Shared registration and Actions remain pending.

- Finish the commit-by-commit audit of all 56 original fix-title commits and
  supplementary behavioral changes; preserve superseded contracts explicitly.
- Add missing complete public outputs for CSS, opaque templates, script block
  identity, directive entities, raw regions and source-owned root tags.
- Keep call-count/performance helper requirements alongside output witnesses;
  a public output snapshot cannot prove the number of formatting passes.
- Register prepared inputs and expected bytes in the shared differential
  runner, with fail-closed execution and exact result accounting.
- Verify full Actions checks and the merge-queue corpus before closing #6882
  or replacing the legacy formatter path.

Other prepared comparisons receive no shared corpus or native acceptance
credit yet. Native formatter support is unavailable; handled, equivalent and
paired native comparisons remain zero. #6882 remains open.
