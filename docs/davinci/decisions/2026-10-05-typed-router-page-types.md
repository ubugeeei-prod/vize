# Configured Vue Router page types (#7817)

The external report [#7817](https://github.com/ubugeeei-prod/vize/issues/7817)
describes page SFCs whose argument-less `useRoute()` retains the union of all
routes. Vize previously did not apply the explicitly configured
`vue-router/volar/sfc-typed-router` plugin's embedded-code specialization.
The reporter is `naitokosuke` (GitHub user ID `102337893`); the meaningful
source commit must retain their verified `Co-authored-by` trailer through the
protected squash merge.
The same prepared scope, findings and unfinished gates are recorded in the
[paired issue decision](https://github.com/ubugeeei-prod/vize/issues/7817#issuecomment-5983222729).

## Supported configuration and authority

This slice implements the known plugin's type specialization natively. It
does not execute arbitrary JavaScript plugins or claim that other entries in
`vueCompilerOptions.plugins` work. A plugin entry must be the exact known
name, either a string or the Vue language-tools `{ name, options }` object.
Unknown plugins, malformed root paths and the unrelated route-block plugin
do not enable the transform. Inherited plugin lists replace earlier lists;
an empty list clears them.

Plugin `options.rootDir` takes precedence over the effective TypeScript
`rootDir`. Vize uses its project root when both are absent, a separately
identified Vize fallback. A filename is a safely quoted relative literal;
the generated router declaration, rather than a filename parser or guessed
route name, owns `_RouteNamesForFilePath` and the unknown-file all-route
fallback. The upstream references are pinned to Router
[`071f1969be1348e797a55d0d8afb72b8068154dc`](https://github.com/vuejs/router/blob/071f1969be1348e797a55d0d8afb72b8068154dc/packages/router/src/volar/entries/sfc-typed-router.ts)
and Vue language-tools
[`7e0d6c4c62d3e0a356968593c34e4aae8e9b9cd5`](https://github.com/vuejs/language-tools/blob/7e0d6c4c62d3e0a356968593c34e4aae8e9b9cd5/packages/language-core/lib/compilerOptions.ts).

## Authored bindings and projection boundaries

Only proved references inside the authored setup block are specialized.
Ordinary direct imports from `vue-router` or `vue-router/auto`, and unbound
auto-imported references, are eligible. Local shadows, aliases, members,
strings and comments retain their original behavior. Explicit arguments or
type arguments remain authoritative.

TypeScript `useRoute()` and `typeof useRoute` receive the route-name helper.
JavaScript `useRoute()` receives a virtual TypeScript return-type assertion;
its authored JavaScript bytes remain unchanged. Non-JavaScript
`definePage(arg)` receives the filename literal, rather than a route-name
type. The official visitor stops inside a matched non-JavaScript
`definePage` argument; JavaScript and unmatched calls retain normal descent.
The template `$route` type is replaced only when the actual template uses it.
The upstream identifier-only visitor's local-shadow behavior is not claimed
as parity: Vize's binding guard intentionally preserves local definitions.

The authored TS/TSX/JS/JSX language controls parsing. AST-decoded names also
cover escaped identifiers; a raw keyword substring is not binding authority.
An edit requires a unique mapped identifier, contiguous authored offsets,
byte-equal generated call/type-query text and safe projection boundaries.
Existing mapping replacement logic splits eligible linear spans and shifts
mapping rows and semantic links. Invalid parse results, ambiguous or
nonlinear spans and unsupported boundaries retain their original generated
code. The existing style-scoped-class append helper moves unchanged into
the existing style helper module to keep each source file within its limit.

## Program visibility and cost

A generated type import may introduce global declarations that are absent
from the authored dependency scan. The registered project records only
actual route-helper imports produced under explicit known-plugin
configuration, and removes that record on replacement or deletion.
When a registered generated route-helper import is present, requested
parallel checking retains one whole native program. Replaying only the old
connected-component plan would still split independent components without
cheap leaves, so it cannot prove equal global visibility. Removing only the
page from a candidate set is also insufficient. This stronger opt-in guard
follows independent source review before publication; its prepared shared-
leaf and independent-component runtime controls remain unexecuted.

Disabled or unconfigured projects, configured but unused files, explicit
calls and local shadows do not invoke this fallback. A secondary synthetic
fixture makes the route-helper import the only source of an otherwise
missing global; it is distinct from the primary router type oracle.
The fallback's cost remains unmeasured and is not a speed improvement.
The separate #7698 whole-command 500-SFC target remains 425.5 ms to
42.55 ms on its original cold workload; demonstrated 10x remains unfinished.

## Fixture evidence and remaining gates

The primary corpus copies the existing pinned Router fixture
`feed382f2fbfe38b3892ea780f5aea3d5459986a` page
`packages/playground-file-based/src/pages/users/[userId=int].vue` and the
entire original generated `src/routes.d.ts`. It must preserve every byte
of that primary declaration. The fixture plugin predates the `typeof` arm;
that arm uses an explicit semantic reference to the pinned newer source.

The primary provider is prepared from the complete official Router 5.1.0
archive, 313,773 bytes, SHA-256
`5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f`.
Its pinned SHA-512 integrity authenticates all 70 original members
(1,167,017 bytes), declarations/chunks, plugin and MIT license. The official
npm provenance statement names source
`c0e3226dabccd7596b996ce851386997ea2d3cca` with that subject digest; reading
it does not claim independent Sigstore signature verification. This source
differs from the page/map and newer visitor pins and is explicitly a mixed
provider chain, never a feed-built artifact. Router 4.5.1 and handwritten
primary declarations are not used.

The bounded standard-library stager rejects changed archives, unsafe/link/
duplicate members, count/size drift, missing exports/license and occupied
outputs before adoption. Seven offline transport test functions passed
locally without fetching or executing a provider. The plugin's actual two
runtime imports use existing frozen-store `muggle-string` 0.4.1 and `pathe`
2.0.3; stable Vue 3.5.35 comes from the existing docs importer. No root
package or lock upgrade is introduced. Real physical package/manifest and
lock identities, the whole archive/member receipt and every original paired
CLI/reference stream are retained in existing differential evidence.

The focused corpus is registered in existing full Check `test-scripts`,
after its actual default source CLI build and genuine receipt. Exact
CLI/editor/native requirements apply to that fixture only. The generic Vue
parity build adds legacy features; its execution must not be falsely
labelled with the default receipt recipe. Provider adoption review and exact
hosted runtime qualification remain pending.

Prepared source tests cover configuration, binding guards, JavaScript,
split script blocks, Unicode, mapping rows, semantic links and conditional
partitioning. The runtime oracle must compare complete ordered CLI
diagnostics/status/report membership, exact editor ranges and clean →
invalid → repair reports against the qualified provider. It must also prove
the secondary implicit-import global visibility and helper-absent controls.
These tests have not yet executed on an exact hosted source.

The secondary visibility controls retain original stdout/stderr bytes,
status and process errors before JSON decoding or report assertions. Their
existing differential artifact also includes exact command flags, binary
identities, the genuine source-build receipt and full authored/configuration
bytes. A failed check therefore remains reviewable after its temporary
workspace is removed. These synthetic controls do not supply primary
Router-provider credit or a performance result. The
[paired transport decision](https://github.com/ubugeeei-prod/vize/issues/7817#issuecomment-5983401125)
also records rejection of empty-segment aliases and file/directory-prefix
collisions before extraction; the same seven offline functions pass with
these additional adversarial subcases, without a provider execution.

Before admission: qualify the prepared provider and
source-built CLI/editor custody, pass fresh exact-source Actions and full
runtime fixtures, preserve all unaffected corpus outputs and unchanged
104 instruction ceilings, obtain protected full suites and actual signed
merge, and include the result in the next verified non-Davinci release.
The existing release/Fuzz queue hold remains in force.
