# Native diagnostic compiler-options custody

Issues: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

## Observed boundary

The audit starts from actual main `6c96b8bb97`, after the signed #7629 related
diagnostic capability fix `29e43907`. The native `OriginalProgramCheck` and
`NativeVueCheck` already retain the complete pull `Full` report: numeric/string
codes, both UTF-16 range endpoints, related information/documents and every raw
field. Their authored spans are separate; the audit found no native raw-field
loss to repair. The historical batch start-only fixture contract is a different
boundary and does not establish complete fix-history acceptance.

Root configuration path/bytes and original source equality do not bind inherited
options. The front API can admit `strict: false`; an inherited `base.json` can
change at the real `--lsp` startup, leaving the source and `tsconfig.json` bytes
unchanged. The diagnosing process then returns TS18047 under `strict: true`
while the old result retains `strict: false`. This is a genuine consumer gap.

## Bounded implementation

The existing original JS/TS/JSX/TSX and native Vue entry points attach the pinned
Corsa 1.14.0 API client to the **same actual diagnosing LSP** using
`custom/initializeAPISession` and its generated Unix pipe. No extra checker
process, dependency upgrade, AST encoder, parser, source scan or level stage is
introduced. The backend creates this API from its existing `project.Session`.
[Pinned backend authority](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go#L1732-L1790).

Before and after the complete raw diagnostic request, a real API snapshot's
`getDefaultProjectForFile` selects the opened original/projected URI. Its full
`ProjectResponse.compilerOptions` must equal the complete admitted
`ConfigResponse.options` JSON value, and its configured filename must match the
actual root configuration. Both fields serialize `*core.CompilerOptions`; there
is no field removal, option filling, root-membership inference or alternate
normalizer. The backend program receives the same command-line options.
[Shared response fields](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/proto.go#L496-L523),
[actual program options](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/project.go#L286-L317).

Success retains the full API-session response, both complete snapshot responses
and selected project responses alongside the untouched raw diagnostic report
and genuine original owner. Their released handles are historical observations,
not live query capabilities. An options mismatch returns the existing typed
`ConfigurationChanged` refusal. SDK targets without real Unix pipe attachment
return `UnsupportedConfiguredProject`; they do not fall back to another process.

Every managed snapshot is explicitly released while the backend is alive,
including on failed project lookup/options comparison. Cleanup reaps the owning
LSP before closing and draining the attached API reader: Corsa's Unix transport
reader keeps a full-duplex socket clone, so attachment `close()` alone can time
out waiting for the peer. Earlier attachment failures still run the outer
owned-LSP cleanup. Existing root/source snapshots and refusal scopes remain.

## Wider snapshot prerequisite remains open

These two actual configured-state observations do not prove the intervening
diagnostic used an identical source/configuration/import graph. A transient
options change and restoration, or an unreported filesystem change, is outside
this bounded guarantee. Empty `updateSnapshot` calls allocate new IDs in this
pinned backend; comparing equal IDs would refuse all ordinary successes.
[API update](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/api.go#L14-L27),
[unconditional snapshot allocation](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/snapshot.go#L325-L497).

Complete graph coherence needs genuine backend prerequisites:

- Bind the pull diagnostic to the actual diagnosing snapshot. Its current full
  report exposes no snapshot/result identity.
- Provide snapshot-bound source-only text/hash receipts for every actually
  loaded original/import/library file. The real `getSourceFileNames` lists the
  retained program's complete files; reading their disk bytes afterward cannot
  prove they equal bytes already loaded by the backend.
- Expose the same snapshot's retained root/extends/package configuration bytes
  and hashes. `parseConfigFile` performs an unversioned fresh filesystem parse;
  a separate parse cannot attest the diagnosing snapshot's closure.

Current `getSourceFile` exposes text/hash only through its recursive full-AST
binary encoder, adding the forbidden extra traversal/serialization. Installed
Corsa 1.14.0 keeps that payload opaque; a 1.14.2 protocol-7/8 decoder is not
compatible with this backend's protocol 5. Neither a decoder upgrade nor disk
hashes are accepted substitutes. These remain real upstream API requirements.
[Snapshot file enumeration](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/session.go#L1079-L1096),
[fresh config parse](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/session.go#L1013-L1041).

## Validation and remaining scope

New source laws require actual unchanged source/root bytes, inherited
`strict: false` to `true` mutation at diagnosing startup, typed refusal and
owned-process reaping for original JS/TS and native Vue JS/TS. An independent
real backend law preserves the complete unexpected TS18047 vector and verifies
the admitted-options mismatch. Positive laws retain the actual full options,
project and snapshot responses, including distinct ordinary snapshot IDs.

Existing ten original Program whole vectors, native Vue vectors, real Vue JSX
and annotated TSX related-declaration vectors, UTF-16 mapping, same-basename
shadowing/profile/module refusal, root mutation and timeout/cleanup laws remain.
Local verification is source formatting/diff only; no local compile/install.
Exact source Actions and protected queue proof are pending at this decision.
All wider graph APIs, complete #6879 history and default migration remain open.
