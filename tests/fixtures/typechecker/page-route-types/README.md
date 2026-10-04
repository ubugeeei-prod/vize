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

Runtime execution and exact hosted source qualification remain pending.
The existing global release/Fuzz admission hold applies. No speed gain or
complete arbitrary-plugin contract is claimed by this fix.
