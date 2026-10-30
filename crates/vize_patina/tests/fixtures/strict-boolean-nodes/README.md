# Checker AST fixture

`conditions.bin` is the unmodified `getSourceFile` response for `conditions.ts`
from the pinned `@typescript/typescript-darwin-arm64@7.0.2` worker. Source path:
`/private/tmp/vize-checker-node-fixture/conditions.ts`.

The encoded tree uses protocol 5 and UTF-16 positions. The checked upstream
encoder is `microsoft/typescript-go` commit
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`, `internal/api/encoder/encoder.go`.
String indexes address pairs in the u32 offset table, not pair ordinals.
Node 22 (kind 213) is the entire `items[0]`; node 28 (kind 214) is the entire
`getTitle()`. Returning these handles to the same worker resolved object and
string types. Tests also reject malformed bounds, strings, source snapshots,
parent links, unsupported protocol versions and truncated payloads.

Source-built CLI and stdio LSP contracts query their own live snapshot nodes;
they do not reuse the fixture's handles.

`tests/_fixtures/differential/linter/native-boolean-tsrs-11147/conditions-tsrs9.bin`
is a second unmodified `getSourceFile` response for the
same authored `conditions.ts`, captured from `tsrs@0.20.1-ts7.1.0-beta` on macOS
arm64. Its package source is `oxc-project/tsrs` commit
`77d3cddb44540734abc33a42ccca4d3f33bee527`; the encoder constants are in
`crates/tsrs_api_codec/src/format.rs`. Protocol 9 has a 64-byte header and the
same 28-byte node records; its optional binder section follows the nodes.

`conditions-tsrs9.json` records the actual image and packet hashes and the
complete `getTypeAtLocations` responses from the same still-live snapshot.
Node 22 (kind 214) resolved the complete `items[0]` to an object, and node 28
(kind 215) resolved `getTitle()` to `string`. The raw packet is not rewritten
for its different header, source-file identity or changed syntax kind values.
The original stable worker capture remains unchanged. Corruption tests mutate
copies explicitly and never count those bytes as observed worker output.
