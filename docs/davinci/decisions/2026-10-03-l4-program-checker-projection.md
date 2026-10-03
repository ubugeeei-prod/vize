# Original Program checker projection

Issues: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

## Authority and admitted source

`targets::ts::project_program` takes only a borrowed completed L2 `FileArtifact`.
Its private `ProgramProjection` retains that same File and its sole internally
minted `ScriptUnit`. The unit must cover every original source byte, have an
actual Module profile and root scope, and coexist with no template operations.
An empty neutral `FileBuilder` has no Program unit and refuses. Partial SFC
script windows, multiple units, nested units, JSX, Script/definition profiles
and incomplete language facts refuse before writing. The caller cannot pair a
detached AST, source window or document with a File to construct the result.

This is a structural L4 provider using genuine existing Program/File APIs.
It does not replace a product path while the #6849 product-order gates #6840
and #6879 remain open. Existing lower admission defines its JS/TS syntax
subset; primitive TS annotations currently remain incomplete and refuse.
The neutral JSX provider can complete a genuine JSX-profile File; this target
still refuses it with `UnsupportedProfile`. The profile law checks both that
lower completion and the exact target refusal, independently of the incomplete
Script-profile and unsupported-syntax laws.
No additional parser, AST traversal, pipeline stage, legacy dependency or
inter-level serialization is introduced.

## Output and checker kind

The existing append-only writer receives the whole authored source once,
followed by generated `"\n;\nexport {};\n"`. The newline ends any trailing line
comment; the separate semicolon protects statement separation. The explicit
empty export preserves the original Module goal for a checking consumer.
Hashbangs, BOM, comments, CRLF, identifier escapes and Unicode stay byte-exact.
The producer does not erase TS or normalize JS text.

`SourceKind::JavaScript` selects an `.mjs` input and requires the consumer's
actual JS ScriptKind/allowJs/checkJs settings. `SourceKind::TypeScript` selects
`.ts` and the actual TS ScriptKind. JavaScript is never silently checked as
TypeScript. The File has no authoritative filename; a later checker adapter
must establish workspace configuration and conservatively handle relative
imports instead of inventing an authored module path.

Recording and non-recording entries have identical text. Only the built-in
`Recorded` and `NoLinks` sinks can mint a projection: a public caller-defined
`LinkSink::into_document` cannot substitute unrelated text into this checked
result. There is no public result constructor or mutable document getter.

## Diagnostic coordinates and links

One complete identity `SpanLink` covers every authored UTF-8 byte. Generated
suffix bytes have no authored link. `map_span` rejects invalid/reversed or
non-UTF-8 ranges, generated-only selections and cross-boundary selections.
A zero-length diagnostic at the authored end maps deterministically to that
authored point; a nonempty range starting there is generated-only.

`map_utf16` converts the actual checker global UTF-16 start/length with the
existing L0 coordinate helper, then applies that exact identity range.
Mid-surrogate positions and checked-addition overflow refuse without clamping.
A non-recording document explicitly refuses diagnostic mapping. A checker
adapter may retain the original backend diagnostic together with a typed
mapping refusal; it must not invent a nearby authored location.

The existing Source Map v3 serializer preserves the original `sourcesContent`
and starting segment. The whole range link proves exact checker offsets;
per-line/per-token v3 anchor coverage is not claimed by this initial slice.

## Evidence and remaining work

Actual L1 stock Program admission feeds L2 `ProgramInput`/`FileProducer`, then
the L4 projection. Laws cover sole owner/real declaration identity, byte/link
parity, original profiles, partial/multiple/nested units, neutral empty Files,
mixed template nodes, parser recovery, equal-copy source refusal, Unicode and
generated diagnostic boundaries. Compile-fail laws protect private construction
and the live File borrow.

The checker law runs the repository-selected TypeScript 6.0.3 on the actual
Rust-generated projection and original source, comparing complete ordered
diagnostic code/category/message/start/length values. JS JSDoc and the same TS
source distinguish checker kinds; Unicode, hashbangs, trailing comments and
multiple diagnostics exercise authored range mapping. The Rust law checks that
the helper returns exactly one result for every fixture before mapping them.
These are native plain
Program input/diagnostic laws, not Vue or fix-history acceptance.

TODO: the opt-in Canon adapter must consume this sealed result through its real
Corsa/TSGO configured transport and preserve complete diagnostics and mapping
refusals. Vue template/Options API/macros/props projection, broader TS/JSX source
admission, complete source maps, fix-history closure and default consumer
replacement remain unfinished. Exact-head Actions, the unchanged all-100 gates,
protected queue checks and actual merge are required for delivery credit.
