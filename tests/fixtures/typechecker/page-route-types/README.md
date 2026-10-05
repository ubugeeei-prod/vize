# Page-scoped Vue Router type oracle (#7817)

The primary page and complete generated declarations are copied at test runtime
from the existing `vue-router` git fixture, revision
`feed382f2fbfe38b3892ea780f5aea3d5459986a`:

- `packages/playground-file-based/src/pages/users/[userId=int].vue`
- `packages/playground-file-based/src/routes.d.ts`

The generated declarations are never replaced with a minimal route model.
They give `userId` the parsed `number` type and supply the actual file-to-route
helper and `definePage` path-key mapping. The helper verifies the pinned Git
fixture before and after execution. TAP observations retain page/declaration
hashes, actual provider hashes, complete CLI output/report vectors and statuses.
The existing source-built LSP session capture retains raw editor traffic.

`prepareProvider()` stages the complete official published Router 5.1.0
archive, rather than claiming an artifact built from the feed fixture. Its
integrity is
`sha512-HAbiLzLEHQwxPgvsbOJDAwtavszEgLwri6XfyrsPECIFez8+59xc9LofWVdc/HEaSRT822lJ8H9Ns38VVond5g==`;
the 313,773-byte archive has SHA-256
`5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f`.
All 70 safe regular members (1,167,017 bytes), original declarations, plugin,
internal chunks and MIT license are preserved and hashed. Seven offline
transport tests reject changed, unsafe, incomplete and wrong-version inputs.

The official npm [provenance](https://registry.npmjs.org/-/npm/v1/attestations/vue-router@5.1.0)
statement names source `c0e3226dabccd7596b996ce851386997ea2d3cca`, tag `v5.1.0`,
and release workflow `26581227939/attempts/1`, with the same SHA-512 subject.
Reading that registry statement is distinct from independent Sigstore
signature verification. This provider source differs from the page/map and
newer visitor pins; the fixture uses that explicitly recorded mixed chain.

The plugin's actual CJS imports are `muggle-string` and `pathe`. The fixture
supplies only their already locked versions 0.4.1 and 2.0.3 from the frozen
workspace store, and stable Vue 3.5.35 from the existing docs importer.
It records actual physical package/manifest identities and the frozen lock
hash; it does not install or upgrade root dependencies. Installed Router
4.5.1 and handwritten declarations are not primary providers. The complete
original archive, member receipt, source CLI build receipt and every paired
raw CLI/reference observation are saved under
`target/differential/router-page-types/`; existing full tooling evidence
upload also retains the source-built LSP's complete raw sessions.

The primary configuration explicitly sets `compilerOptions.rootDir` to the
playground root. An object plugin option overrides a deliberately conflicting
compiler root, and an inherited compiler-root case checks effective paths.
Vize's project-root fallback uses only `project-root-map.d.ts`, a separately
identified control extension. Its vue-tsc reference sets that root explicitly;
it is not claimed as upstream's no-root behavior.

The pinned plugin predates the `typeof useRoute` arm. Those tests compare the
explicit generic type-query transformation described by source revision
[`071f1969be1348e797a55d0d8afb72b8068154dc`](https://github.com/vuejs/router/blob/071f1969be1348e797a55d0d8afb72b8068154dc/packages/router/src/volar/entries/sfc-typed-router.ts).
They are a semantic reference to that arm, not parity with the feed382 plugin.
Local shadows likewise use a disabled-plugin reference: the upstream
identifier-only visitor would rewrite them, while Vize preserves their local
bindings. All other listed controls use the real provider and generated map.

The oracle checks clean → invalid → repair CLI and editor reports for same-line
diagnostics after the inserted type argument, template `$route`, `definePage`,
JavaScript and `typeof`. It checks complete ordered diagnostic vectors,
authored start/end ranges, program membership and exact repaired CLI reports.
Explicit arguments/generics, disabled configuration, unmapped files, normal
script, member calls, aliases, comments and strings are separate controls.

The focused action is registered in the existing full Check `test-scripts`
job, after its default source CLI build and genuine build receipt. It sets
exact CLI/editor/native requirements only for this fixture and hydrates the
unchanged pinned Router gitlink. The generic Vue parity build has additional
legacy features, so this test does not mislabel that build with the default
receipt recipe or broaden other fixtures' launch policy.

The new secondary `partition-controls.ts` is explicitly synthetic. Its helper
is excluded from registered roots and alone supplies a global type; it checks
shared-leaf and independent/no-leaf programs, disabled/explicit/shadow/no-call
and absent-helper guards, exact 1/2-server ordered reports and virtual output,
and unrelated ambient type errors. It is not primary Router-provider credit.
`partition-capture.ts` saves complete original raw streams and authored/config
bytes before decoding/assertion under `target/differential/router-page-types-secondary`;
failed runs retain the genuine source-build receipt and exact binary/flag identities.

The primary observer also saves original product/reference process bytes before
JSON parsing or process-error throws, then restores authored source and config
bytes in `finally`. Five transport tests with three nested cases use only owned
Node stubs to check binary/nonzero/ENOENT output, inert error descriptors, failed
capture, malformed JSON and reference restoration. These controls do not run
Vize, the native provider or the official Router plugin.

Complete primary runtime qualification remains pending.
The existing global release/Fuzz admission hold applies. No speed gain or
complete arbitrary-plugin contract is claimed by this fix.

The provider is physically stored in the isolated workspace's
`node_modules/.router-page-provider/package` and linked as `node_modules/vue-router`.
Vize classifies physical package stores outside `node_modules` as workspace
source packages; the first hosted replay of this oracle intentionally exposed
all seven runtime JS families as checked workspace sources. Keep the original
archive/export/declaration bytes and `checkJs`, rather than filter those results.
The complete report root vector also includes the original configured
`node_modules/vue-router/vue-router-auto-routes.d.mts` declaration before the
unchanged authored include vector. This remains configured root metadata,
not a native/transitive graph-closure claim.

Only the original page and generated map are copied from the pinned playground.
The selected builtin-int entry declares `userId: number` literally. The map
retains references to unselected custom-parser sources that are not assembled
here; these observations do not prove the complete upstream project graph.

The typechecker editor oracle explicitly disables the separate ecosystem group
(route/i18n heuristics and optional completions) in its saved config and
initialization options, alongside lint.
It keeps native typecheck, complete unfiltered diagnostic vectors, authored
UTF-16 ranges and hover assertions enabled. The original page's query keys
produce two route-heuristic warnings under the editor bundle; those genuine
raw warnings are retained as failed evidence, not discarded from an accepted
payload. Production defaults and heuristic behavior remain unchanged. This qualifies
the explicit typechecker profile, not default-editor parity.

The original multiline `definePage` negative remains strict: unknown path keys
must produce the full reference TS2353 diagnostic and authored range. Setup
code generation indents body lines, so filename-only macro specialization
validates its exact mapped callee/header and preserves the generated body.
Real registered-project controls retain whole body bytes and diagnostic map
endpoints. Matching original-length multiline header prefixes preserve all
argument bytes; mismatching prefixes decline without editing. This does not
prove equality through the actual generated argument start. Fresh hosted
native replay is required; partial earlier cycles are not complete parity.

The one `define-page-unknown-id` rendering fixture records independently reviewed
full native and full official-JS TS2353 messages for the exact pinned source,
original map and provider archive. Both complete vectors and common ordered
fields remain strict; every other message comparison is unchanged. This is
not a general message equivalence rule. The full LSP vector must equal the
native vector; originals remain captured before comparison. See the
[rendering decision](../../../../docs/davinci/decisions/2026-10-05-typed-router-diagnostic-rendering.md).
