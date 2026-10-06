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
Admitted JS, TS, JSX and TSX blocks use their authored parser mode. Mismatched script languages,
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

Fresh successor `24c3dc82e0` no longer panics in actual authored-source lint.
[Job 112110895370](https://github.com/ubugeeei-prod/vize/actions/runs/37414781119/job/112110895370)
then reports exactly two real setup-time URL reads in the playground's
`getInitialTab()` and Musea's `getTabFromUrl()`. The bounded consumer repair uses
`globalThis.window?.location.search` in their existing URLSearchParams calls:
browser query/tab behavior stays the same and an absent SSR window yields their
existing default tab. The two files keep their previous line counts, including
Musea's grandfathered 366 lines. The zero-warning gate and rule/corpus assertions
stay unchanged. This [paired correction](https://github.com/ubugeeei-prod/vize/issues/7982#issuecomment-6009503285)
requires fresh successor Actions; the failed old job does not qualify it.

Independent successor source review closes the first two lexical defects but
rejects `24c3dc82e0`'s space-only block padding: a trailing authored `//` comment
can swallow the other real script. One LF now occupies the private padding byte
immediately after each script content span. Authored bytes and all physical
reference offsets stay intact. The complete diagnostic controls supply exact
source-witnessed descriptor frames in both comment/read ownerships and physical
orders; they qualify that private boundary mechanism, not the legacy SFC parser's
acceptance of a closing tag inside a trailing line comment. Actual tooling job
`112110999581` also rejects the existing generated census's stale SfcDescriptor
non-product row (13/19 versus 14/20). The regenerated current row includes the
new typed-frame test references (15/21), without changing its generator, baseline
assertions or public API. These failed receipts require fresh source proof.

The same source's four Rust workers execute 19 distinct rule laws: 18 pass and
the four-language loop fails with warning count 0 versus its required 1. The
original whole issue and documentation diagnostics and lexical controls pass,
but this is not whole-source acceptance. Preserve all four original language
inputs and expected findings; add only language/error/diagnostic context to that
assertion so the next hosted result identifies the failing arm. Whole SFC
admission for those cases remains unfinished; no local parser/native probe or
weaker expectation replaces the genuine failure.

## Actual TSX boundary failure and signed-main incorporation

Source `2d7901367b` passes authored app lint, Clippy/build and all four tooling
workers. Its four Rust workers execute 20 distinct rule laws: 19 pass, including
the supplied-frame LF control, while the unchanged language loop identifies
`lang=tsx` and `parser/sfc`: "Malformed <script> block: the closing tag is missing."
[Actual failed job](https://github.com/ubugeeei-prod/vize/actions/runs/37416588029/job/112117854230)
retains the complete diagnostic and required warning count 1; no expectation is
removed. All raw worker logs remain separate from successor qualification.

The existing SFC regex lookahead starts at JSX `</p>` and mistakes the real
`</script>` slash plus invalid `script` flags for a regex terminator. A private
wrapper keeps that scanner's original result except for this actual closing-tag
endpoint in explicitly authored JSX/TSX; rejected lookahead restores its prior
line state. Normal JS/TS and genuine regex/class contents retain the original
result. There is no extra parser, AST walk, pipeline stage or public API. Retain
both exact language inputs in a new SFC differential corpus, preserve the old
whole language/CLI/LSP assertions, and add full descriptor/content/location,
comparison-regex and other-language controls. Three new parser laws remain
unexecuted locally; default instruction cost is unmeasured, with unchanged
protected ceilings required before admission.

Security #8080 actually merges as signed valid `48cb1d4f35`, sole parent `143c1d4`,
at 2026-10-06T05:04:12Z. Incorporate that literal whole main once, preserving all
incoming production/development provider pins, lockfiles, UI fixes, workflows and
canonical clauses. Old source audit failures do not qualify this new tree; fresh
source Actions and the unchanged actual CLI/Nuxt/LSP/pinned SSR/full/protected
qualification remain required. Root's finite release selection is v0.435 minor;
actual source cut and published verification stay pending.
