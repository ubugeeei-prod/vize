# SSR browser globals in authored scripts

Issue: [#7982](https://github.com/ubugeeei-prod/vize/issues/7982).

The rule's own Bad example was not reported. The retained implementation
visited interpolation and directive expressions, but had no full-SFC script
callback. The original `WidthLabel.vue` therefore reported only the template's
`document` access and missed setup's `window` and `navigator` reads.

The regression corpus retains the complete original 178-byte SFC, 96-byte
configuration, issue body and both original documentation examples under
`tests/_fixtures/differential/linter/ssr-script-setup-7982/`. Its manifest hashes
are authored input custody, not observed product output. The actual reporter
is `ubugeeei` (public account ID `71201308`); the commit credits that identity.

## Decision

Add a private script check to the existing full-SFC rule callback. Reuse the
shared descriptor and the established rule-local OXC/semantic pattern used by
`vue/no-mutating-props`. Croquis' existing browser list is narrower and its
stored uses do not establish this rule's direct `typeof` or deferred execution
policy, so treating that list as complete would lose necessary distinctions.
There is no new public option, Croquis field, pipeline stage or native provider.

The pass exits before parsing when the rule/SSR mode is disabled or neither
script contains a candidate browser name or identifier escape. Candidate
scripts incur one private syntax/semantic pass. This cost is unmeasured;
no default speed or typechecker improvement is claimed. Template scanning was
moved into a private module in a separate move-only commit; its logic and
existing template messages, directive behavior and snapshots remain intact.

Both authored script blocks retain their original physical byte ranges in
one padded parse. Before building semantics, the private setup statements move
into an anonymous child function scope; actual setup imports stay in the module,
and repeated imported source/local/name bindings preserve Vue's first import.
Normal-script bindings remain visible to setup in either physical order, while
setup locals cannot shadow module reads or leak into a called module function.
No authored reference span or public AST changes. Block/source equality, language,
external-source and cross-block statement guards refuse an unsupported Program.
JS, TS, JSX and TSX use their authored parser mode. Mismatched script languages,
setup exports and experimental import attributes/phases are not traced. Malformed
syntax or semantic diagnostics produce no additional script-rule findings;
ordinary diagnostics remain owned by their existing paths.

Only value references without a runtime binding to the rule's existing browser-global list
are reported. Lexical bindings, parameters, hoisted declarations, property
names, strings, comments, regexes and type-only queries remain quiet. Direct
`typeof name` is safe. Exact equality/inequality with the string `"undefined"`
protects the branch where that global exists, including conditional/logical
expressions. The probe must resolve to that runtime global; a block-local same-name
probe cannot suppress an outer function's global read. A member probe or the
opposite branch remains reportable.

Function bodies are deferred unless directly called, called as a witnessed
local function or passed to a named Vue import of `onServerPrefetch`,
`watchSyncEffect`, or default-options `watchEffect`. Imported aliases preserve
symbol identity; unrelated same-named functions cannot forge a Vue callback.
Mounted and event callbacks remain deferred. Class instance fields remain
deferred while class static initializers run immediately. Recursive/repeated
calls retain one finding per physical reference. This is a bounded heuristic:
external/namespace callbacks, reassigned functions, generator iteration, parameter defaults, instance construction and
Options API methods are not traced, and the rule is not a proof of SSR safety.

## Acceptance and unfinished work

Prepared Rust laws retain whole original rule diagnostics, byte spans and
suppression/disablement; exercise actual shadow/hoisting, split script identity,
types, ambient/type-import erasure, guards, deferred/SSR callbacks, recursion, languages and escaped Unicode
identifiers. They have not been executed locally; Actions owns execution.

The source-built CLI fixture validates the genuine default build receipt,
retains complete raw status/stdout/stderr before assertions, checks whole JSON
and plain original outputs with incremental and no-config Nuxt modes, and checks
both documentation examples. Two actual stdio sessions assert complete
incremental/Nuxt publications across original, unsaved repair and restoration;
the original disk bytes remain unchanged. The existing transport retains its
framing, deadlines and bounded raw streams. Native typechecking is disabled in
these sessions: they qualify lint behavior only.

A separate existing pinned Vue 3.5.35/plugin 6.0.7 SSR fixture records full
compiled graphs and actual rendering outcomes for the original issue,
server-prefetch access and deferred mounted/event callbacks. That fixture
tests the execution assumption; it earns no Vize native compiler or typechecker
credit. Local work is formatting/static review only, with no native build,
provider execution, installation or instruction measurement.

Fresh exact-head source Actions, authentic full CLI/LSP/runtime fixture results,
unchanged protected 100+4 instruction ceilings/full suites and actual signed
merge remain required. Incorporate the actual signed security repair main once
it lands before protected admission. Keep any failed receipt separate from
successor qualification. Published original verification belongs to the next
finite existing-product release; Davinci, fix-history closure and 10x remain
unfinished.

## First-source rejection and narrow successor

Paired [issue correction](https://github.com/ubugeeei-prod/vize/issues/7982#issuecomment-6009449929).

Draft source `4d11de9c34` is rejected, not an accepted fix. Independent source
review found two lexical ownership defects: a bound local `typeof window` probe
incorrectly protected a called outer function's global, and flattened script
blocks let setup locals hide module globals. The successor retains semantic probe
identity and the private module/setup frame relation described above, with complete
positive/negative diagnostic controls in both physical orders.

Actual [Check 37412867906](https://github.com/ubugeeei-prod/vize/actions/runs/37412867906)
then exposed an OXC declaration-node panic in authored source lint and the hydrated
glyph corpus. This consumer now explicitly requests the AST node store its symbol
and ancestor lookups require. The same run rejected wildcard/unwrap Clippy usage
and stale generated linter import inventory; explicit imports, fallible language
selection and regenerated inventory preserve existing policies. The separate
security failure belongs to the still-pending security repair baseline. All raw
failed job logs remain retained; none qualifies the successor or a runtime result.
The original whole CLI/LSP/SSR assertions and all instruction budgets stay intact.
