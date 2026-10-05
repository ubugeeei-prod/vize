# Native bulk diagnostic categories

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
This is a private dependent source preparation on the explicit Batch config
binding in #7857. Its parent delivery and finite release cut do not wait for
this optimization. Source and runtime qualification are still pending.

## Protocol authority

The actual runtime is TypeScript native 7.0.2 at
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`. Its
[GetDiagnosticsParams](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/proto.go#L1107)
admits an omitted `file` for syntactic, semantic, suggestion and declaration
categories. The Program collects its complete member set, sorting and
deduplicating within each category. The pinned SDK remains corsa_client 1.14.0:
its existing `raw_json_request` and numeric-handle normalization call these
actual methods. Its grouped diagnostic method names are absent from this
native protocol and do not establish a supported bulk endpoint.

## Admitted ownership

Only a nonempty Batch with an explicit config is eligible. Reuse its existing
configured API attachment to the same diagnosing LSP process. Synchronize the
complete existing document map through the original recovery helper, retain
the response-backed generation barrier, then obtain one fresh managed snapshot.
Require exactly one snapshot project, its exact configured path, and every
requested physical source in the Program's complete source-name result.
Noncanonical URI aliases refuse this route so grouping cannot produce a false
empty result. No new process, parser, attachment, persistent snapshot, SDK
version or product pipeline stage is introduced.

Use the same snapshot for all categories. Return requested URIs in their
original order, including duplicates and successful empty members. Preserve
syntactic, semantic, suggestion, then optional declaration order without a
cross-category sort or deduplication. Declaration runs when normalized
`declaration` **or** `composite` is true, matching native GetEmitDeclarations.
Global, config and program diagnostics are separate endpoints and are not
invented additions to per-file LSP parity.

Project UTF-16 offsets using only acknowledged owner-retained source text.
The native [LSP line map](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/lsconv/linemap.go#L21)
counts CRLF, CR and LF; Unicode separators are ordinary LSP characters.
Preserve complete message-chain indentation, numeric codes, category severity,
all eight native style-warning codes, source `ts`, and related-information
order, coordinates and unflattened localized messages. Existing client
capabilities advertise related information and no diagnostic tags or Visual
Studio extensions. Complete internal diagnostics are cached before the
unchanged public projection.

Release each managed snapshot eagerly on success, refusal and error while its
owner lives. Existing owner-before-attachment shutdown and retry bodies remain
unchanged. Unsupported methods, ambiguous projects, unowned related sources,
invalid positions or fileless category rows refuse the whole batch; partial
bulk vectors never enter the diagnostic cache. Transport failures propagate
through the existing recovery path. The default unconfigured editor domain and
the existing non-Unix attachment refusal are unchanged.

## Cost and remaining gates

Project-wide collection checks the entire Program even for a small requested
subset. Conversion builds one position index per acknowledged document and
buffers returned categories. A late custody refusal adds this work before the
existing fallback; fewer per-file calls alone do not prove a latency gain.
The response-backed dirty-document readiness work is deliberately retained.

The retained eight authored profiles in
`tests/_fixtures/differential/typechecker/native-bulk-diagnostics/profiles.json`
exercise strict/loose options, styles, syntax, checked JS, declaration,
composite and Unicode/CRLF related diagnostics. New native laws compare complete
preprojection vectors with the same process's original per-file LSP results for
initial, unchanged and edited generations, including duplicate URIs and an
empty member. Test-only receipts retain the actual successful category
snapshot, project, source names, method list and attachment. The existing eight
generated-config laws and other original qualification commands remain intact.

Pending: technical source review, exact-head Actions, real native responses,
full original 500 warm/no-op/leaf/shared-dependency and CLI/LSP/config/delta
vectors, declaration controls, matched source timings, protected full suites,
and actual ordered native Stack merges. No new performance result, 10x claim,
default migration, native Davinci product completion, history closure or budget
increase follows from this private preparation. #7698 remains open.
