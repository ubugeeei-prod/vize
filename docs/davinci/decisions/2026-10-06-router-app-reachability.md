# Installed app route reachability (#7932)

Decision: [public issue receipt](https://github.com/ubugeeei-prod/vize/issues/7932#issuecomment-6018050334).

The literal Nuxt report has a `useRouter()` component and an unrelated closed,
unnamed router exported by its component test. The CLI adds both to the same
supplied project. The old consumer treated every discovered router as reachable
from every app injection, so the test tree falsely proved that `users-id` was
absent. This path has no Nuxt pages or configuration provenance.

An injected navigation now needs a proven application context. The proof uses
scoped runtime Vue `createApp` or `createSSRApp`, an imported default SFC root,
and exactly one unconditional top-level `.use(router)` with a directly resolved
router. The root's relative runtime imports and that router's recorded component
imports establish the supported context within the supplied project. Vue aliases
and namespace constructors retain binding identity. Type-only bindings do not
establish runtime ownership. Shared modules retain independently proven contexts;
an unknown context refuses the app claim. Unknown roots conservatively refuse
all app claims. Factories, conditional/logical installation, unknown plugins,
dynamic installation and multiple router plugins on one app remain unknown.

The decision follows the actual Vue Router v4.5.0 `install(app)` implementation,
which provides the injected router when `app.use(router)` installs it:
[primary source](https://github.com/vuejs/router/blob/v4.5.0/packages/router/src/router.ts),
blob `748a06a32a2adbd558ddde432cd3823e650c0df6`, SHA-256
`d2803cbfbf2cf3608874fbbc3266dba7a7d5b0213d0c32a1b38f80ed682b6461`.
History choice and test filenames do not establish app ownership. Direct
`Target::Router` selection, route facts, open-tree refusal, diagnostic text,
parameter checks and witness verification remain unchanged. Valid SSR memory
routers are still checked when installed or directly referenced.

This intentionally uses the report's permitted conservative result: the Nuxt
page table remains unknown without genuine application proof. It does not invent
page inference, read private files, parse framework configuration or add a
provider, pipeline stage, serialization boundary or dependency. Each candidate
script in the new collector is parsed once; its import adjacency is retained
for all app-context traversals.

The new differential corpus retains the complete public issue, exact component,
test and command fences, and the inline config. Page bodies were not provided;
the two minimal page carriers are separately labeled authored. Twenty whole
CLI cases cover the original report, installed root/component reachability,
unrelated/orphan routers, direct-router unknown/missing/extra/type findings,
absent/dynamic/conditional/logical/factory/ambiguous/shadowed refusal, SSR memory
routing, independent apps/projects and shared contexts. Entire file inventories,
messages, severities, ranges, help, stdout, stderr, status and source bytes are
asserted from authored expectations. Failed observations must be retained before
the final aggregate assertion.

Every original carrier and golden in the five existing route corpora stays
byte-exact. Supplementary authored app roots/bootstrap files supply the real
installation context that the old injected-navigation controls assumed. Their
original diagnostics, spans, facts, witness chains and undeclared-access laws
remain required. These additions are explicit coverage context, not rewritten
reporter inputs or recaptured golden outputs.

Source preparation is not runtime qualification. Fresh hosted CLI/Rust execution,
all existing route laws, full protected suites and instruction gates remain
required before actual signed delivery. No performance, native-default, Nuxt
page-provider, installed-public or release result is claimed here. Root owns the
subsequent supported release and public consumer handoff.

A bounded private source review found that body-only function nesting misses
default-parameter initializers and that an uninstantiated class expression can
carry an instance-field initializer. The complete Function, Arrow and Class
walks now carry the existing nested-context refusal, including parameters and
fields. The existing semantic ScopeFlags re-export supplies the visitor signature;
there is no new dependency or traversal pass. Three whole refusal controls join
the unchanged original seventeen cases, for twenty cases and ninety-four full
file rows. This source correction is paired in
[the same issue](https://github.com/ubugeeei-prod/vize/issues/7932#issuecomment-6018277539).
Original inputs, expectations and source pins are preserved. Runtime remains
unqualified until fresh Actions execute this exact successor.

The first hosted exact-0a83 Check failed strict Clippy at the adjacency indexing
expression before Rust/CLI tests executed (run 37478294823, job 112319688990).
The complete failed raw log remains retained. Checked mutable access now retains
the valid project's complete imported edges; an impossible invalid module index
conservatively refuses all app contexts. Direct router checks are unaffected.
Every input, expected vector, old golden, budget and provider policy stays exact.
This meaningful same-PR correction is paired in
[the issue](https://github.com/ubugeeei-prod/vize/issues/7932#issuecomment-6018409224).
The authentic red caused Draft/offqueue containment. Fresh corrected source
Actions must own qualification; no earlier or unrelated green is transferred.
