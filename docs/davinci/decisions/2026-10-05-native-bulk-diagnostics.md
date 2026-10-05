# Native bulk diagnostic categories

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
This dependent source slice builds on the explicit Batch config binding
in #7857. Its parent delivery and finite release cut do not wait for
this optimization. Bounded technical source review passed; runtime
qualification is still pending.

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
URI membership compares exact strings derived from native source names; Path
equality would normalize interior `.` and repeated separators, although row
keys preserve lexical bytes. Dot, duplicate-slash and percent aliases refuse
this route so grouping cannot produce a false empty result. No new steady-state
process, parser, attachment, persistent snapshot, SDK
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

Attempt eager release of each managed snapshot on every outcome while its owner
lives. A legal deeply nested native message chain can exceed the pinned SDK's
raw JSON reader envelope, which closes that attachment and converts the pending
error to Protocol. Matching Json alone or swallowing failed release cannot make
that reader healthy. The typed private route distinguishes complete, bounded
refusal and required owner retirement. A request or release failure retires the
failed owner's process using the existing lifecycle helper, then the entire
original LSP diagnostic path recreates and acknowledges a healthy owner; its
flattened messages do not have the same nested JSON shape. Retain real request,
release and retirement causes if that whole fallback also fails. Local payload
shape refusal does not invent upstream Unsupported provenance. Existing
owner-before-attachment shutdown and retry bodies remain unchanged. Unsupported
methods, ambiguous projects, unowned related sources, invalid positions or
fileless category rows refuse the whole batch; partial bulk vectors never enter
the diagnostic cache. Only a complete category result credits NativeBulk;
successful original LSP fallback credits Editor. The default unconfigured editor domain and
the existing non-Unix attachment refusal are unchanged.

## Cost and remaining gates

Project-wide collection checks the entire Program even for a small requested
subset. Conversion builds one position index per acknowledged document and
buffers returned categories. A late custody refusal adds this work before the
existing fallback; fewer per-file calls alone do not prove a latency gain.
Failure recovery additionally rebuilds the owned process. The response-backed
dirty-document readiness work is deliberately retained.

The retained eight authored profiles in
`tests/_fixtures/differential/typechecker/native-bulk-diagnostics/profiles.json`
exercise strict/loose options, styles, syntax, checked JS, declaration,
composite and Unicode/CRLF related diagnostics. New native laws compare complete
preprojection vectors with the same process's original per-file LSP results for
initial, unchanged and edited generations, including duplicate URIs and an
empty member. A separate retained legal `deep-return-chain.ts` input requires the actual
native reader refusal, complete old/new-owner LSP equality, and a later
acknowledged bulk generation on the same healthy successor attachment. The
original `deep-chain.ts` remains byte-exact as a positive collapsed-property
control. Fixture depths are witnesses, not production input budgets. Test-only
receipts retain the actual successful category
snapshot, project, source names, method list and attachment. The existing eight
generated-config laws and other original qualification commands remain intact.
Complete sides and custody are archived before comparison assertions, including
failures; no failed raw vector is regenerated as an expected result.
Test-only custody resets for every request, including disabled observation and
unconfigured or empty batches, so a previous successful category packet cannot
be attributed to a later original LSP fallback.

## First hosted qualification

Source `426a2453377b3cb1ea6774ecfef0a7a5b6d1c4ee` compiled on Actions.
Native run `37306128972` passed eight unit laws and the eight-profile law:
24 phase packets contain 72 complete original/bulk row comparisons and 78
acknowledged category calls. Its authenticated artifact `11344635748` has ZIP
SHA-256 `198bfaa8ae9a55779f94bcd98c90def98497ae621143337e2ccd29d3356575f8`.
The recovery law failed because the original 80-property input produced a
successful, equal bulk result. The pinned
[Relater reduction](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/checker/relater.go#L4824)
collapses adjacent property errors into one dotted path. Preserve those exact
input bytes and failed runs; their success does not prove reader retirement.
The successor adds a nested return-signature witness without that property
reduction and retains the mandatory actual reader-failure/recovery assertions.
The first source-length gate also failed
because module registration grew `diagnostics.rs` from 498 to 499 lines; the
same-change repair keeps the parent cap without changing the existing body.

Successor `82b2d7ac55f63d73ac0f576d1941f2c81a7a1203` passed native run
`37308278705`: eight unit and three native laws, including the mandatory actual
reader refusal and recovery. Artifact `11345262019` has ZIP SHA-256
`24c8cf3f492e3aabb58c895d09ebb5b3be886a8c484b5ae8fb254db49be8cb99`;
all CRCs passed. The return-chain packet records real Protocol recursion refusal,
attempted release returning Closed, successful owner retirement, complete
old/new-owner diagnostic equality, a different attachment pipe, and a later
acknowledged bulk generation on that same healthy successor. The original
property control stays on its existing owner with equal complete vectors.
All four source Rust workers passed. The source Check still failed on one
extra Markdown blank line; this receipt corrects it. These are exact `82b2`
receipts, not qualification of a later parent replay or matched timing.

## Original Tier-L admission qualification

Genuine parent replay `c6b538228ca56dc42c14f8708dd839a6d2d8ad93` passed
[native run 37311368805](https://github.com/ubugeeei-prod/vize/actions/runs/37311368805):
eight unit and three native laws. Its original 500 SFC / 681-file
[Vue parity job](https://github.com/ubugeeei-prod/vize/actions/runs/37311684779/job/111768388780)
passed the unchanged cold/warm, single-session and one-file-delta gates.
Artifact `11345964859` has ZIP SHA-256
`1548f1fc8c6788983aaa5ffe15ea4560bab572c6315a95403005c6d9bbc87196`;
all CRCs passed. The single unpaired sample is 8552 / 6750 / 6951 ms.
These are quality observations, not a matched speed result or proof that
project-wide categories supplied those 681-file responses.

The existing `VIZE_NESTED_BATCH_CAPTURE_DIR` receipt is compiled only in lib
unit tests, so it cannot attest the integration timing build. Its nested
`App.vue.ts` selector also does not represent the original Tier-L inputs.
The successor therefore adds a separate ignored lib law in the existing
hydrated Vue parity action, after the unchanged integration timing test.
Both tests share the original fixture path selection, 500-root count,
153-byte clean and 160-byte broken injected sources, and file cleanup.
The pinned fixture revision, nested configuration, 681-file request census
and all timing budgets remain unchanged.

For cold, broken and repaired generations, the law observes the existing
materialized session and compares every complete cached native diagnostic
with the original per-file LSP path before public projection. It preserves
all requested URIs, generated files and source maps, generated configuration,
both full vectors and actual route/attachment custody before assertions.
A bounded refusal now retains its reason in test-only evidence; it earns
no `NativeBulk` credit. All three generation packets are saved before route
or vector assertions, so a refused first generation cannot hide the later
original cases. This law must demonstrate successful categories and exact
whole vectors on the real inputs before the optimization can be admitted.
No production observation flag, readiness bypass, provider or budget change
is introduced. Matched same-host timing remains a separate pending gate.

Source `774e2918b46e34d45bcb495519dd75fa6884d8d4` compiled the new ignored
law, passed affected Clippy and all four PR Rust workers, and passed native
run `37314846723` (eight unit / three native laws). Its tooling shard 2 failed
only because relocating the original suffix membership assertion to the shared
fixture helper exposed an unlisted partial assertion. The repair requires the
complete `fixture_root.join(INJECTED_FILE)` path to be present, strengthening
fixture custody without changing source bytes, selection, budgets or the
committed allowlist. The authoritative Rust assertion gate reruns on the
successor. Whole-681 execution in full run `37315591587` failed actual bulk admission;
these source/native results do not prove real Tier-L bulk admission or gain.

## Actual original Tier-L refusal

[Full run 37315591587](https://github.com/ubugeeei-prod/vize/actions/runs/37315591587/job/111781417778)
passed the original integration timing test, then failed the mandatory ignored
law at source `774e2918b46e34d45bcb495519dd75fa6884d8d4`. Artifact
`11348018551` (8527295 bytes) has ZIP SHA-256
`4aaeb1f89264a3a95d70354ce0e076ccc42527c770caad6d709985774e3eaaa8`;
all CRCs passed. Every cold/broken/repaired packet contains 681 requested rows,
and all 2043 complete fallback rows and public preprojection rows equal the
original LSP vectors. The actual route is `editor-lsp`, with acknowledged
snapshot release and bounded conversion refusal. This is sound fallback
custody, not successful project-wide admission. Later LSP checks in that job
were skipped after the real failure and receive no fresh qualification credit.

The original `preferences.vue.ts` TS2769 vectors include related locations in
the pinned native `lib.es2017.object.d.ts` (line39, columns4..36) and
`lib.es5.d.ts` (line261, columns4..30). Those unrequested sources are absent
from the acknowledged document-text ownership used by this converter.
One configured project and exact requested membership passed; conversion is
the observed refusal boundary. Do not remove related information, force a
NativeBulk marker, weaken the whole-681 law or read changed disk bytes after
freezing the snapshot to claim custody.

The pinned native `getSourceFile` endpoint returns a snapshot-owned encoded
AST. Existing [SDK1.14 `get_source_file`](https://github.com/ubugeeei-prod/corsa-bind/blob/34bb731bdcaf5d4c154596b0c32912cbffd60b18/src/core/corsa_client/src/api/client.rs#L403) exposes an
opaque [`EncodedPayload`](https://github.com/ubugeeei-prod/corsa-bind/blob/34bb731bdcaf5d4c154596b0c32912cbffd60b18/src/core/corsa_client/src/api/encoded.rs#L15);
it has no decoded text/line-map provider, and no such provider was found in
the current repository. An invented AST decoder is outside this slice's
no-new-adapter scope. A supported snapshot-text ownership seam or a separately
reviewed ownership design is required before extending conversion to these
stdlib related sources. The current prepared optimization remains Draft and
offqueue; the independent parent release cut does not wait for it. Matched
same-host gain remains unproved and #7698 remains open.

Pending: exact-head Actions, fresh native responses after parent replay,
full original 500 warm/no-op/leaf/shared-dependency and CLI/LSP/config/delta
vectors, declaration controls, matched source timings, protected full suites,
and actual ordered native Stack merges. No new performance result, 10x claim,
default migration, native Davinci product completion, history closure or budget
increase follows from this prepared source. #7698 remains open.


The authentic whole681 refusal retains its complete original vectors. A small snapshot-text prerequisite is now actual Draft #8049 below this child in GitHub native Stack #8051 (positions 1→2), with the historical merged #7857 prefix preserved in #8039. The consumer must replace its direct managed-snapshot creation with that sealed provider on the same retained configured API, use its one original ordered source-name response and exact project/config/URI admission, and request absent related text only during the existing conversion. Cache positions once per related URI. No later disk read, second parse, additional diagnostic walk, new process or SDK/default/readiness change supplies the witness.

The private consumer preparation retains Complete/Refused/RetireOwner and every real request/release cause: typed unsupported/malformed/foreign source text refuses the entire category result, actual SDK error attempts release while the owner lives and retires it before the entire original LSP retry. Keep all complete category ordering, duplicate requests, empty rows, UTF-16 coordinates, message chains, style severity and related information. Two independent whole-row controls cover repeated unicode/CRLF related sources, fetch-once behavior, successful empties/duplicates and whole refusal/error after a prior row has already converted. Every old unit/native/reader-recovery fixture remains unchanged.

Test-only custody retains each actual related source's complete opaque bytes, complete text, URI/fileName/path and same snapshot/project before any final comparisons, including bounded refusal. The original full681 law additionally requires the two authentic stdlib witnesses and exact owner/source-name identity; its mandatory NativeBulk/full-vector assertions and original three full generations remain intact. Provider native proof passed at exact0f743481/run37330415540 with the authenticated full artifact, seven pure/three lifetime/one actual Linux native laws and full pre-snapshot stdlib/overlay equality. The private consumer connection remains uncompiled/unexecuted; matched original500 warm gain, CLI/LSP/cross-config/delta/declaration qualification, full source/protected suites and actual signed delivery are still mandatory. No 10x or default/Davinci completion claim follows from the private connection.
