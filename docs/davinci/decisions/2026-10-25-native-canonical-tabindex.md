# Shared native canonical tabindex query

Issue: [#10489](https://github.com/ubugeeei-prod/vize/issues/10489).
Producer: [#10484](https://github.com/ubugeeei-prod/vize/pull/10484).

`NativeCanonicalLint` borrows only an admitted original `NativeSfc` assembly or
the same-Program `JsxMarkupFile`. A raw Artifact, File, rejected owner, or caller
span cannot construct it. The context retains the actual canonical File borrow.
The opt-in `a11y/tabindex-no-positive` query makes one shared canonical traversal;
it does not parse, decode, lower a compatibility AST, or copy a frontend rule.

The existing i32 predicate is shared with both registered rule entries and the
original-header native API. The completed L0 diagnostic constructor is shared
with that original-header API. Native producer-decoded static attributes retain
their full original ranges; expressions are not interpreted as static strings.
Diagnostic order follows authored ranges, and every returned message is owned.

Original JSX Program comments refuse context construction, including comments
whose empty JSX containers do not become canonical ops. Vue markup comments
refuse the whole query before pending findings escape. Source-block offsets are
projected from the genuine unit, including nonzero UTF-8 prefixes. These explicit
refusals preserve the host suppression boundary. Configured suppression adoption
and the production linter driver are separate unfinished work.

Eight ordinary laws compare 125 complete native/registered-rule packets across
Vue, JSX and TSX, all three locales, both quote forms, LF/CRLF, Unicode, integer
limits and invalid values. Requeries preserve the entire ordered output. Vue's
original decoded-reference cases are separate from JSX's explicit normalization
refusals. Original incomplete Files and native expression holes remain retained
refusals. The 31 existing original-header laws also pass unchanged.

The bounded `patina.native.canonical.tabindex` source span measures the shared
query. A reproducible ignored probe uses 65 canonical ops, 64 children and 16
complete findings, with 500 calls per input. On local macOS arm64 in the
development profile, lazy per-query catalog capture changed observed average
query time from 40.603 to 29.720 microseconds for Vue and from 40.420 to 29.608 for
TSX. The logarithmic p95 upper bucket changed from 64 to 32 microseconds. Catalog
lookup work is pinned at two calls per positive query and zero for a clean query.
Preparation and result destruction are outside this window. These local source
observations do not establish an optimized runtime or official instruction gate.

Run the ordinary laws with `cargo test -p vize_patina --test integration
native_canonical_tabindex`. Run the single timing probe with that test name's
`timing::canonical_tabindex_source_phase_measurement` suffix and
`-- --ignored --nocapture --test-threads=1` in a quiet process.

This adds one genuine native multi-input consumer. It does not select the default
product route, admit Pug or other producers, establish whole-product history
acceptance, or complete #6848/#6871. Exact-head Actions, protected merge and
publication remain independent delivery evidence.
