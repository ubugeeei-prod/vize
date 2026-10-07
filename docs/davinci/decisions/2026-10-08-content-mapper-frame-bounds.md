# Bound Content Mapper frame headers

Owning issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).

The standard `vize content-mapper` stdio reader previously bounded only the
message body. `read_until` could allocate an unlimited header line, many short
headers could accumulate without a total bound, and a repeated Content-Length
silently replaced the first length. These are source-owned framing stability
gaps, independent of the native TypeScript semantic and declaration gates.

The reader now limits each frame's complete headers to 8 KiB, including every
line ending and the terminating blank line. A limited read enforces the remaining
allowance before an arbitrarily large line can be accumulated. A complete header
block at exactly 8 KiB is accepted. An exhausted allowance without a complete
terminator is rejected without consuming body or subsequent-frame bytes.

A second case-insensitive Content-Length is rejected before its value is parsed,
whether it repeats, contradicts or invalidates the first length. The existing
64 MiB body bound, CRLF/LF handling, unknown-header handling and JSON-RPC project
and transform semantics remain unchanged. Fragmented reads retain Interrupted
retry behavior through the standard library's bounded reader.

The regression corpus retains fourteen whole raw stdio inputs with independent
input and complete response hashes. Public CLI tests compare exact stdout,
stderr and exit status for three write fragmentation sizes. Controls cover clean
EOF, concatenated CRLF/LF frames, the exact header boundary, one-byte and many-line
overflow, equal and conflicting duplicate lengths, missing/invalid lengths,
truncated headers/body, the unchanged body ceiling, and a valid response before a
later fatal frame. Reader laws additionally retain UTF-8 body bytes, arbitrary
internal buffer fragmentation, interrupted reads and precise consumption bounds.

A focused standalone harness extracted from the actual old reader demonstrates
two failing framing laws; the changed reader passes those laws and healthy
controls. This is local reader evidence only. The source-authenticated public
CLI and full existing protocol tests require fresh exact-head Actions, followed
by root-owned protected qualification and actual merge. No historical or local
binary substitutes for that evidence; no instruction ceiling is raised.

The complete tsconfig graph, declarations, native mapper semantics, LSP parity,
installed-consumer replay and the typechecker's 10x speed target remain unfinished
where their own required witnesses are absent. This bounded change does not
close #3984 or qualify n8n JSONC/project transport. The separate original-source
JSONC authority proposal remains read-only with its bridge owner; upstream n8n
and TypeScript stay read-only.

Initial source `b4e26ecd71c7d3202a20ed36f924364a956725f7` failed Check `37653924137` at its Rust test build, before any reader, CLI or lifecycle execution. The installed sha2 0.11 digest array does not implement LowerHex. The helper now converts each byte with `{byte:02x}` while retaining all fourteen original inputs, complete stdout/stderr/status expectations and hashes. No production reader, old protocol vector, instruction ceiling or workflow changes. Fresh successor Actions remains mandatory; historical success or skipped workers supplies no execution credit.

Repaired source `75ceef1207ac998070901d6b9c24d6f17ed2e1a0` passes its Rust build/clippy but Check `37654714645` tooling shard 3 fails the unchanged content-mapper consumer-surface assertion: nine preferred imports were expected and the new reader test directly introduced a tenth. The reader test now reuses the parent module's existing preferred L0 `cstr` import. Regenerated inventory returns to the original nine; the complete existing assertion and every original framing input/output/hash remain unchanged. The focused original surface test passes locally. Fresh current-source Actions remains required; no original oracle, dependency rule, instruction ceiling or workflow is relaxed.
