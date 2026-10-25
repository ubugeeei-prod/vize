# Native compiler diagnostic DTO boundaries

The per-file compiler-kind adapter recognizes the native 7.0.2 diagnostic shape
and the known additive fields from TypeScript 7.1.0-beta, pinned to
`microsoft/TypeScript@3e9236361fa97a9e6c685f4833ac457ae2d8c50a`.
The primary definitions and producers are `tsc/internal/api/proto.go` and
`tsc/internal/ls/lsconv/converters.go`; the published SDK declares the same fields
in `dist/api/proto.generated.d.ts`.

`pos` and `end` remain absolute UTF-16 code-unit offsets. Beta additionally emits
`startPosition`, `endPosition`, and `sourceLines`. These positions use its
ECMAScript line map, including U+2028 and U+2029. The adapter checks the complete
metadata against the exact source text from the same owned snapshot. It checks
the producer's first-two/last-two context selection for long spans and preserves
all original line endings in comparisons. LSP output uses its separate
CR/LF/CRLF line map. A custom diagnostic `source` is preserved; the empty prefix
uses `ts`, matching the native pull converter.

Omitted additive fields preserve the older DTO. A partially supplied location,
explicit null location, malformed field, unknown field at any decoded level,
foreign source or contradictory text rejects the complete read. Diagnostics are
never discarded to make an incompatible response look clean.

The existing API handshake supplies the request dialect. SessionSnapshots keeps
7.0.2's singular `file` request unchanged. DerivedSnapshots uses `files: [file]`,
as required by beta. The adapter requests a single file directly and verifies
returned source identities. `getSourceFile` retains its singular file parameter.
No version request or additional snapshot is created for this selection.

Source text has an explicit binary format boundary. Protocol 5 retains its
existing reader. The separate protocol 9 text-root reader uses the beta encoder's
64-byte header, 28-byte node envelope, source kind 308, and 76-byte SourceFile
extended envelope. It bounds the selected root, all string pairs and known root
references, checks exact source identity and UTF-16 source length, and validates
the selected strings as UTF-8. AST, binder and content-mapper payloads remain
opaque and provide no diagnostic coordinates. Unrecognized versions refuse the
text view.

The checked-in protocol 9 fixture was obtained from the actual official beta
Darwin arm64 image, with independently authored Unicode/CRLF/LS/PS text and a
same-snapshot semantic diagnostic. Its JSON sidecar records image, payload and
raw-stream identities. It is an ABI fixture, separate from source execution.

The dedicated `Native beta diagnostic DTO` workflow runs the identical public
observer against pinned before and candidate source roles and independently
pinned Linux images for 7.0.2 and 7.1.0-beta. The before role receives only the
separately hashed test observer and its test-only module registration; its
production bytes remain the pinned source. Candidate private decoders are never
copied into that role. The corpus retains complete semantic-error, repair,
hover and reference packets, an erroneous neighboring file, held-snapshot
reads, exact source bytes and closed native API/LSP streams. The before beta
role must retain its strict DTO failure; candidate beta must complete the
same-snapshot adapter. Each SDK/source pair retains its own provider and source
receipt. This qualifies the bounded diagnostic adapter and its exercised
features, rather than general beta editor or resolver support.
