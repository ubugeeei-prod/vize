# Original Options API instance diagnostic requirements

Tracked in [#6879](https://github.com/ubugeeei-prod/vize/issues/6879), a Stage 0
foundation with no provider dependency. Product replacement via #6849 and native
typechecking remain unfinished. This independently reviewable fixture adds no
normal level-to-legacy edge and changes no production checker or product path.

## Original requirement and options

Fix [#6680](https://github.com/ubugeeei-prod/vize/pull/6680), immutable commit
`35bdad84760e6251edd52bb2e3e52701974a9d63`, preserves unchecked template `$props`
when Vue resolves an Options API instance to `any`. The original project combines
a loose `ComponentOptionsMixin` declaration with a concrete `defineComponent`
mixin; inferring `$props` from `any` previously produced `unknown` and TS18046.

The new pack retains both exact original projects from that commit. The regression
project must return no diagnostics in any file. Its typed-props control keeps the
complete original two TS2339 tuples at `src/OptionsInvalid.vue:13:18` and `14:23`,
including full messages, code and severity. The valid SFC in that same project
must add no diagnostics. Expectations come from the original entire-vector
assertions; they are not a freshly invented or self-derived diagnostic baseline.
Historical assertions sort the vector. Fresh required execution must verify that
the current complete returned order matches the retained expectation before
runtime admission; no sorting or message normalization is added to the new test.

Both original bodies call `BatchTypeChecker::enable_options_api()` before scan and
check. The serialized pack and shared case rows explicitly retain `optionsApi=true`.
The mandatory runtime body enables that same public option. Old eight packs retain
their existing implicit default options and every original input/oracle byte.
Changed or omitted options fail even if an altered pack's hashes are recomputed.

The five SFC/TS inputs and original `tsconfig.json` literal are byte-exact carriers.
The provenance records original commit, Git blobs, SHA256, complete test-source
carrier, exact helper excerpt and UTF8 literal offsets. The original runtime/Vue
guards give no new execution credit. Current required execution links the pinned
real Vue package and fails on absent runtime, construction, scanning or checking.

## Whole observation and required verification

The existing source-built Rust integration test and complete-pack observer execute
each original project once, compare every authored start/code/message/severity/file
row in returned order, then retain the entire actual public batch result. Absolute
paths, zero-based starts, optional block type, exit code and success remain observed;
only the original diagnostic projection earns matched-contract credit. End ranges,
related information and raw backend payloads remain unavailable.

Registration extends the corpus from 19 projects/74 input references to 21/81, in
nine mandatory bodies. One exact T0 deferral is added for this body; the full
required profile remains unfiltered with real TSGO and retries zero. Existing
four-worker reconciliation requires all nine successful historical JUnit identities,
exact source/tree, archive receipt, extracted executable, fixture and input hashes.
No expected-only object, synthetic admission law or old executable grants runtime
acceptance. Fresh exact-head Actions and actual protected queue/merge remain required.

The previous eight-body/19-project foundation actually merged in native Stack #7375
on 2026-10-01 at 19:39:41 UTC, with all 19 original diagnostic projections matched.
All 19 native projects remained unsupported with handling/equivalence/paired counts
zero. That terminal evidence does not execute these two new original projects.
Their current source-built Actions captures are pending; whole semantic history,
native typechecking, full-range/raw fields and product replacement remain unfinished.
The ledger marks this original fixture prepared rather than historically admitted.

## Next native typechecker hook

The first native comparison waits for the genuine L3 typed-scope provider and L4
`ts` target, with the same immutable native L2 owner carried through analysis and
emission. The scope and expression owners must supply actual source-owned script
and template bindings, dialect facts and access policy; the emission owner must
retain authored links in `EmitDocument`. This fixture's caller option supplies no
binding or scope facts, and its legacy checker remains a differential oracle.

Once those providers exist, the first bounded hook checks these exact two projects
and original options through the native virtual-TS target, runs the real backend,
and compares the complete diagnostic projection plus retained actual public fields
and authored mappings. Missing native scopes, `any`/typed-prop semantics or target
support remain typed unsupported outcomes. No legacy analysis, synthetic fixture
bindings or legacy product route may satisfy that native hook. Provider source,
Actions observations, full-history admission and #6849 product replacement remain
separate required work.
