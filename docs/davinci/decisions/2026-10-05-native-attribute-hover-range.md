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

## Qualification and remaining work

The local closed-input/comparator controls and configured type-aware lint pass.
Rust projection tests and all twelve real native replies are unexecuted at
source preparation; exact-head Actions must establish their results. Queue
admission is held during the finite first v0.433 publication. After fresh
source acceptance, protected suites, actual signed merge and supported public
verification remain required. This correction supplies no new native-level,
default-migration or whole fix-history completion credit; #6883 stays open.
