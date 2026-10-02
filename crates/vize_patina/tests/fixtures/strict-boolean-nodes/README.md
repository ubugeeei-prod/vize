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
