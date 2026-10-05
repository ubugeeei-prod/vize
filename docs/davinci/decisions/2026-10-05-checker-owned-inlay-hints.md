# Preserve checker-owned variable type inlay hints

Issue: [#8004](https://github.com/ubugeeei-prod/vize/issues/8004).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8004#issuecomment-5993319002).

The original SFC and literal Vize configuration are retained under
`tests/_fixtures/differential/lsp/computed-inlay-hints/`, with hashes and the
reported Vue 3.5.41/version/environment. The tsconfig fixture reconstructs the
fields supplied in the report; omitted compiler options are not claimed as
original bytes. The frozen workspace lock already contains Vue 3.5.41, so the
fixture selects that exact installed package rather than the Playground's
different Vue release. It does not change workspace dependency links or locks.

The old reactive formatter substituted `_` for an unknown syntax-inferred type.
The actual LSP handler now obtains native variable-type hints through one
standard range request on the existing reusable editor session. The initialization
preference is the supported `inlayHints.variableTypes.enabled` field. Other
diagnostic/style preferences, ATA policy, configured project authority, readiness,
recovery and native request generation/cancellation ownership remain intact.

Pinned primary sources are TypeScript 7.0.2 commit
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`:
[preference field](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/lsutil/userpreferences.go),
[initialization configuration](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go), and
[checker hint production](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/inlay_hints.go).
Native `GetTypeAtLocation` and its alias-aware type printer supply structured
label parts. Vize neither parses hover strings nor reconstructs generic types.

Preserve labels, kinds, padding, tooltips, commands and opaque data. Map positions
and writable edits through complete length-preserving authored spans. Generated
scaffolding, unsupported edit spans and out-of-range hints are declined. Optional
label locations use the existing canonical dependency projection; unmappable
private locations are omitted without replacing the native label. Prop and
translation decorations remain document-only. When native types are unavailable,
the server emits no guessed reactive type. The existing document-only compatibility
helper retains known syntax refs but declines unknown placeholders.

Whole structured projection controls cover UTF-16/CRLF, alias locations, writable
edits, commands, tooltip/padding/data retention and generated-span refusal. Actual
CLI stdio tests compare whole vectors with an independent stock native process on
the retained script bytes, including original/changed/repaired buffers, ranges,
plain JS/TS and generic aliases. Only known corresponding source-file identities
and physical dependency identities are normalized; label fields and ranges stay.
The sole stdout reader captures every decoded RPC envelope before enqueue,
including unconsumed notifications and parser/EOF errors; these are not raw wire
frames. Original inputs/configs and whole native/actual vectors are saved before
URI normalization. After both readers join, each process retains exit status and
raw stderr, plus the executable hash, version output and harness source identity.
Both native and Vize shutdown must succeed. These captures run on Actions.
The stock oracle initializes with the pinned required nullable `processId` and
declared pull-diagnostic capabilities. Before opening a document, it asserts and
acknowledges the complete pinned configuration-watch registration request, which
the native initialization handler awaits synchronously. Its retained bare script has two unused
binding suggestions for `label`/`items`, whose uses occur in the SFC template.
Assert that complete native diagnostic vector separately from the complete clean
SFC diagnostic vector. Authored plain JS/TS controls consume their computed value;
no suggestion/style preference is disabled and no diagnostic is filtered.
Checker-disabled and genuinely unavailable-runtime original RPCs are retained. Existing prop/i18n/resident
laws and the entire legacy differential corpus remain required.

The existing native reference/CSS guard is faithfully extracted before adding
mandatory hint qualification. It retains every original command/status guard;
no phase sampling, instruction ceiling or cold/warm timing budget changes.
The parent configured-session and private bulk qualifications must be retained
when replaying onto their actual merged main, with all source caps at 350.

Source review, exact-head native Actions, protected full suites, signed merge
with the verified reporter trailer and external release inclusion are pending.
This record makes no measured latency, Program-count or 10x speed claim.
Native File and the broader LSP/typechecker history gates remain unfinished.
