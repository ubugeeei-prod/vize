# Native attribute hover ranges

This decision pairs [#7993](https://github.com/ubugeeei-prod/vize/issues/7993).
The reported Vize 0.432.0 response has the correct DOM property type for
`:for` and `:id`, but exposes the separately generated TypeScript document's
range. The reported attribute-value hover already has the correct Vue range.

The complete original 132-byte `Field.vue`, 111-byte tsconfig, 3,682-byte client
and full issue body are pinned in
`tests/_fixtures/differential/lsp/native-attribute-hover-range-original/`.
The reporter's public identity is `ubugeeei`, GitHub user 71201308. The report
uses TypeScript 7.0.2 and Vue 3.5.43; its table provides type summaries, not
complete native hover contents. No historical full contents oracle is invented.

## Source correction

`hover_html_attribute_with_corsa` makes a DOM-property query in a separate
virtual document and previously returned its range through the unchanged
`convert_lsp_hover`. Carry the existing attribute lexer's authored byte span
after only `:` or `v-bind:` normalization, then convert its two endpoints using
the existing physical UTF-16 position helper. Only this native-attribute route
replaces the range. Complete converted contents and the attribute-value route
remain intact; the old tuple lookup keeps every name/tag result. Transformed
model properties without an authored-name span omit the range rather than
expose synthetic coordinates. No public range clamp, new parser/pipeline,
backend query, per-node IPC, native provider or dependency is added.

The reported physical ranges are `for` 5:10–5:13 and `id` 6:10–6:12. The
positive value `id` remains 6:14–6:16. CRLF retains those UTF-16 positions.
Authored Unicode variants put astral characters before both attributes and
retain their independently authored complete vectors. Rust projection controls
also retain complete markup/string contents and cover longhand/static names,
value isolation and unprovided transformed-name spans.

## Mandatory current-source observation

The existing shared fourteen-session LSP corpus intentionally disables type
checking. Its complete objects, initialization, vectors and old positive
responses remain unchanged. A separate closed typed corpus runs alongside it
in the ordinary source tooling selection and full protected suite:

- Four fresh serial source-built `vize lsp` sessions: original LF, original
  CRLF, authored Unicode LF and authored Unicode CRLF.
- Three original roles per session, in order: `for` name, `id` name and the
  already-correct `id` value. All twelve complete JSON-RPC replies are required.
- Original physical tsconfig bytes remain unchanged. The current controlled
  profile explicitly enables editor/hover/typecheck and disables lint; this
  differs from the report's client with no explicit options and a fixed sleep.
  Exact-version diagnostics replace that sleep. Record actual native/Vue
  providers and hashes, without assuming they equal the reported versions.
- Bind the executable to the existing mandatory source-build receipt, literal
  checkout revision/tree and independently hashed executable bytes. Retain
  complete initialization, diagnostics, request/reply envelopes, raw framed
  transport, stderr, physical inputs/configuration, owned PID and exit status.
- Preserve every attempted session and failed/missing response. Compare only
  after all four captures are written. Complete native contents for each role
  must match across all four physical variants, with the reported native type
  signature and independently authored physical range. The first contents
  vector is a current observation, not an original historical oracle.
- Whole comparisons reject nulls, omissions, extra fields, wrong IDs/positions,
  altered full contents, truncation, missing sessions and failed cleanup. Pure
  synthetic frames exercise only this comparator and provide no runtime credit.

The bounded raw report is
`target/differential/native-attribute-hover-range.json`, included by the
existing always-upload corpus artifact. No workflow dispatch or local native
build is needed. The unchanged shared corpus, all full Rust/native suites and
104 instruction ceilings still require genuine protected execution.

## Source-qualified observation and integration

Exact source `80f91e240ae6090b95550f25903a35ee0f7d7759` passed ordinary
Check 37327874770: twenty-five successful jobs and sixteen intentional policy
skips. Its four current typed sessions produced all twelve complete replies
and physical LF/CRLF/Unicode ranges, with identical complete contents per role.
The actual provider was TypeScript 7.0.2 with Vue 3.6.0-beta.10, not the
reporter's Vue 3.5.43. All 16,176 source JUnit cases and the four projection
laws passed without failures, errors or skips. This is source acceptance;
protected 104 ceilings, current integration and actual merge remain separate.

#8047 corrects component-attribute query authority and explicitly excludes
model attributes from the non-model lookup. Its original
`de901695cd49591f2a06b5b2054dd43be24507d4` was used only for the private
source rehearsal. Its signed actual merge is
`94c13ebf4b83bf7549cfcbacff7513c9c4646f36` at 2026-10-05T16:38:46Z.
This successor genuinely integrates literal actual main
`9fe172ced209720642d7051c8a94229042cffa4c`, including that merge and the
accepted warm-input and diagnostic-action corrections. First move the original attribute lexer
into `definition/helpers/attributes.rs` in a move-only commit, preserving its
complete scanner bytes and the existing helper paths. Then combine the same
scanner's authored spans with #8047's model-exclusion flag. The ordinary tuple
and authored-span routes include models; the separate non-model route refuses
them. Accepted transformed models still have no authored span. Static,
shorthand and longhand names retain their original span; directive, dynamic
model and value refusals remain intact. No parser, query, pipeline stage,
backend operation or public clamp is added. Both source modules stay below
350 lines without weakening a cap.

A fifth independent Rust law covers four accepted model forms, three normal
name forms, two invalid models and three refused directives. It checks the
ordinary tuple, non-model exclusion and exact authored bytes together. The
original four whole projection laws, twelve-response corpus, source pins and
shared fourteen-session corpus remain unchanged. The integrated scanner and
five-law source are byte-exact the independently reviewed private rehearsal;
all other actual-main source and incoming canonical clauses are preserved.
Fresh exact-source Actions must qualify this genuine composition before queue
admission. Previous `80f91` results never transfer to the integrated source.

Protected suites, actual signed merge and supported public verification remain
required. This correction supplies no new native-level, default-migration or
whole fix-history completion credit; #6883 stays open.

## Retained initial integration failure

The first exact integrated source `ff5dc737` failed Check 37343248398 in
Build affected Rust tests, job 111875583094. Strict Clippy rejected only the
new authored guard's `source.clone().into()`: its source is already a standard
String, so the conversion is useless. Remove that identity conversion,
without changing any scanner/projection code, guard, input or expected vector.
The failed run provides no complete source/Rust acceptance, and the five
combined laws still require actual fresh execution. Also genuinely integrate
the accepted doc-only main `58e6a027`, preserving its full delivery ledger and
canonical clauses. Fresh successor Actions, complete typed sessions and
protected unchanged104/actual delivery remain required; no lint allowance,
rerun waiver or previous green transfer follows.

## Current accepted-main composition

The genuine signed main `6b5d6a77` contains the accepted SSR builtin writer,
declared event completion precedence and two linter corrections. Integrate
that actual main in the same PR, retain all foreign production, fixtures,
providers, instruction budgets and canonical clauses, and preserve every
reviewed hover scanner/projection/input/whole-response object from `8cd025f9`.
Resolve only the central-record append conflict. The prior synthetic source
checkout `ecc66f34` has a distinct tree containing the accepted SSR changes;
its twelve typed replies are source-bound observations, not whole-tree
equality or acceptance for this successor. Fresh exact-source Actions and
protected typed twelve-response/full-corpus/full-Rust/unchanged104 evidence
remain required before actual delivery.
