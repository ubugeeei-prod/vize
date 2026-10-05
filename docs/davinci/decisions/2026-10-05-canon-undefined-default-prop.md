# Optional callable props with undefined defaults

Issue: [#7819](https://github.com/ubugeeei-prod/vize/issues/7819).

The parser retains the exact authored `undefined` default expression. A later
imported-default resolver also wrote the string `undefined` into prop metadata
as a marker for a proven non-undefined default. The template model treated
every retained default as defined, overriding the existing literal-undefined
filter and adding both a `Pick<Props, ...>` default and an `Exclude<..., undefined>`
assertion. Conditions on optional functions then produced false TS2774.

Retain the existing resolver's proven default-name set separately from authored
expressions, using its already-built set without another source scan. Template
default substitution uses those names and actual non-undefined authored
defaults. Macro bindings consume the same normalized set. A literal undefined
default therefore preserves the callable's optional type, while a real
imported callable default still resolves as defined. The change repairs the
current product and claims no native-stage acceptance.

The original SFC and tsconfig are pinned under
`tests/_fixtures/differential/typechecker/undefined-default-prop/`.
Source-built CLI tests compare complete diagnostic vectors for the original
valid conditions, genuinely defaulted imported callable calls, and rejected
unguarded calls on an undefined default. Independent typed macro/template
statements use the installed Vue declarations with official TypeScript 7.0.2;
all native stdout/stderr and statuses are compared with authored expectations.
The real editor diagnostic service must return the complete empty vector,
and the real native hover service must retain the exact optional callable
signature and authored range.

The native-phase Actions explicitly requires these CLI and editor tests,
retaining full input/output, official package/version and source/binary hash
receipts. Ordinary PR Rust's native-disabled runs are insufficient. Fresh
exact-head source/native Actions, unchanged protected full suites and
instruction ceilings, actual merge and published consumer proof are required.
Queue admission remains under the release owner's publication barrier.

The first required run failed closed before product checks because workspace root
has no direct Vue dependency. Create isolated cases beneath the existing pinned
`npm/cli` workspace package, so both independent native oracle and source CLI
resolve its unchanged real Vue declarations without installation or symlinks.
Pin the editor runtime to the same authenticated oracle binary, retain complete
editor fixture/config inputs and full responses before assertions. This repairs
qualification only; original corpus and all expected diagnostics remain intact.

The next run authenticates the real Vue oracle and reaches all CLI controls.
CLI stderr deliberately includes two existing progress lines; assert their exact
input counts and isolated root instead of incorrectly expecting silence. Full
stdout diagnostic vectors, statuses, raw stderr and original inputs remain
required and unchanged. This is an expectation repair without output filtering.

At 36c6, all three complete native/CLI vectors and the real editor diagnostic,
optional-hover and range assertions pass. Receipt collection must enumerate the
three explicit test names separately from the dedicated editor-inputs bucket;
retain that bucket and hidden fixture bytes in the artifact. No runtime source
or expected response is changed. Fresh corrected-head receipt/source gates are
still required before readiness.

Required b6b0 native Actions pass the exact three real-Vue CLI/native cases
and complete original editor diagnostics/optional hover. Strict source Actions
correctly detect the generated Croquis consumption table changing one macro
usage (72 to 73); regenerate it with the official generator and preserve the
new API semantics. Fresh successor source/native gates remain required; the
receipt is evidence for b6b0 only and protected merge/release is unfinished.

The source/native-qualified eaa3 head cannot enter the queue because its
canonical paragraph conflicts with fresh main. Replay all six commits onto
actual main a2712e7896, retaining their reporter trailers and every owned
production/test/original fixture byte. Put the complete owned clauses beside
the existing type-check decision paragraph; every incoming canonical line is
preserved and the document remains 350 lines. Fresh replay qualification and
protected actual merge are still required; earlier eaa3 green is not reused.

The actual protected 969f candidate failed its fourth Rust worker: the unchanged
imported runtime props fixture gained TS18048 for `title` and `pagerCount`, whose
original defaults are `'Ready'` and `7`. Imported runtime prop resolution uses
`default_value: "undefined"` as a presence/Boolean-cast marker. Preserve that
existing runtime facts only from the already-resolved imported prop vector,
retaining names in the same private resolver-proven set through the generalized
`resolved_prop_defaults` accessor. Raw authored undefined remains excluded;
inline runtime props cannot acquire imported sentinel provenance. The existing
type-based resolver returns before changing runtime facts. No cache, new field,
source input, golden diagnostic or compiler stage changes.

The existing required native qualifier additionally executes all three original
`imported_runtime_props` tests with its same official 7.0.2 binary. It retains
complete cargo stdout/stderr/status and pins the immutable fixture's SHA256;
the source fixture itself preserves every original input/config and each whole
empty diagnostic assertion. The original three CLI/native vectors and full
editor optional-hover response remain required. The failed queue candidate was
automatically removed. Fresh source/native Actions, protected full/all104, actual merge
and publication remain pending; historical 5b green does not cover this repair.

A bounded independent correction review confirms imported-identifier provenance,
type-based early-return retention and unchanged raw-undefined exclusion. Broader
pre-existing imported/runtime default-undefined inference remains unclaimed.
The official consumption generator records the added existing Croquis macro
access (73 to 74); no counter or instruction limit is weakened.

The release owner thawed existing qualified admissions. Literal composition
with #7861 exposed two separately added artifact hidden-input flags at different
positions. Use one shared trailing flag, preserving all raw hidden captures and
commands; no source/native acceptance transfers from the previous head. The
original runtime/CLI/native/editor contracts remain mandatory before admission.

The unchanged Canon runtime test helper and production resolver read CORSA_PATH,
whereas the editor control reads VIZE_TEST_TSGO_PATH. Bind both to the already
verified official oracle binary in this qualifier and retain the runtime source
SHA/binary path in its hashed capture. Existing test helpers, fixture bytes and
whole vectors are unchanged; only fresh corrected-head execution can qualify
the explicit binding.

Artifact76 retains all44 raw hashes and the explicit native binding, but the
original common snapshot helper can return None on constructor/scan/check errors
and the legacy fixture then skips its golden assertion. Preserve every original
input/config string and all three whole-empty goldens, while adding a required
snapshot-presence guard and capture before cleanup. Retain each complete snapshot,
all authored src/packages/tsconfig bytes and direct Vue/Vite declarations. The
common helper stays unchanged; required execution now fails on absent snapshots,
and the receipt independently asserts every complete empty vector. The harness
source hash changes explicitly; original input/config/golden bytes remain exact.
Fresh source/native proof is required before admission, with no skip waiver.

Actual586 source Check37273533198 and native37273532707 both succeed.
Artifact11329513840 independently retains 199 verified raw hashes, all 14
original authored strings, three complete non-null empty runtime snapshots,
three complete CLI JSON/native text expectations and the original full optional
editor hover with exact source/config bytes. Replay onto signed main7c87
repairs only canonical composition, preserving all 13 owned non-document files
and every incoming350-line prefix. The replay requires its own fresh Actions;
no historical runtime receipt transfers to protected admission.

The admission successor rebases onto actual signed main d8c3 and moves the complete
owned canonical clauses to the existing script-side sentence. The earlier own
UNMERGEABLE entry behind #8020 was immediately removed; queued branches are not
source parents. Original source, all whole native/editor vectors and three
mandatory runtime snapshots stay byte-exact. Fresh Actions and clean prospective
ordered queue composition are required before readmission.
