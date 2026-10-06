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

## Authentic full replay and repair-control correction

Paired [issue correction](https://github.com/ubugeeei-prod/vize/issues/7982#issuecomment-6010621282).

Source `229c254524` passes [ordinary Check 37417967216](https://github.com/ubugeeei-prod/vize/actions/runs/37417967216),
including all 20 original rule laws and three parser laws. Its existing
[full Check 37419939190](https://github.com/ubugeeei-prod/vize/actions/runs/37419939190)
retains two separate incoming rich-hover/inlay failures; their actual signed
repair-prefix main incorporation remains required before fresh qualification.

The owned full tooling job executes all six exact JSON/plain CLI observations,
including the Nuxt preset and documented Bad/Good examples. Incremental LSP
passes the complete original/unsaved-repair/restored publications. Nuxt LSP
passes the original three browser warnings, then correctly rejects the repair
control's `{{ 'healthy' }}` with `vue/no-useless-mustaches`; its restore is not
executed. Replace only this repair input with the existing safe `lang` binding.
Keep all original inputs and complete empty/restore assertions unchanged: no
diagnostic filtering, severity change or weaker expected vector repairs the test.

Official artifact `11392794207` is 59,057,643 bytes, SHA-256
`b6bb7336f73dfd34d688f68ae088c98484ce48bf9490b1aa52a9746cdf13bf7e`.
The failed setup receipt (`8dd90d585c034fcd1f0de62b84d188ee3f1af4287ccedcc602454efd8aff1641`)
retains all 13 rows and both graceful bounded sessions (13 client/11 server
frames). The pinned SSR receipt (`a6a77d5b4303051a067838ac5542ba25005aa786a59e4912b54b9f709d88e4c0`)
passes all three cases, retaining complete graphs/maps and the original actual
setup ReferenceError, render warning and TypeError rather than claiming a clean
original render. These are partial product/execution receipts, not whole-source,
protected instruction, native typechecker, merge, release or performance proof.

## Actual rich/inlay repair-prefix incorporation

Paired [issue decision](https://github.com/ubugeeei-prod/vize/issues/7982#issuecomment-6010840948).

The native Stack prefix actually merges both layers as signed valid commits:
`a344f3d32f` for #8095 and `29724e2952` for #8082. Incorporate signed actual
main `890c3ce480` once, whose sole parent is `29724e2952`; it also contains
the delivered #8093 default-preset correction. Preserve every incoming source,
workflow, lock, provider and default-rule byte. The only merge conflict is the
shared canonical line: keep the whole incoming file and append the owned SSR
entry, retaining 350 lines. Regenerated inventories preserve incoming rows.

The private Nuxt input correction and all original 23 rule/parser laws, issue
bytes and complete original/empty/restore expectations remain intact. Fresh
automatic source Actions and one necessary existing full Check must qualify this
new composition; historical `229c` failures and ordinary green remain separate.
Protected 100+4 measurements, full suites, signed merge and an actually included
finite published cut stay pending. Root's v0.435 release work is separate from
this unqualified source; no inclusion, public or performance credit is assumed.

## Independent actual-main delivery after inherited Vapor cap failure

Paired [current-main decision](https://github.com/ubugeeei-prod/vize/issues/7982#issuecomment-6013257289).

SSR source `64ed0ce8a5` passes ordinary Check 37425200171 and full Check
37427289673: all 23 rule/parser laws, six complete CLI observations, both whole
original/repair/restore LSP cycles and all three pinned Vue SSR controls. These
remain historical receipts; they do not qualify the following composition.

Protected candidate `09b6aa99c1` inherits parent `b5f41c39bd` and fails three
`atelier_vapor_lower_large` samples at 79,683 > 76,975. That parent already fails
at the same value; preceding `3c29fb6bbc` passes. Retain all 312 exclusive Ir
samples, original fixture/window/allocator markers, both bounded official ZIPs
and unchanged budgets. The actual instruction job fails; the superseded aggregate
is separately cancelled only after removal from every current queue projection.
The PR is promptly dequeued. The later #8101 repair's protected failure is also
separate; no green source check substitutes for protected measurement.

The SSR source has no dependency on ordinary Vapor reactive-key production.
Its 33 owned paths exclude Vapor/Core/instruction-harness/budget changes, and the
original CLI/LSP lints and pinned stock Vue SSR controls do not invoke that
lowering path. The #8101 owner independently confirms the source separation.

Incorporate actual signed main `b96599a160` once into this same PR. It excludes
unmerged #8101; the fresh live queue excludes that failed slice too. All production
merges cleanly. Resolve only the shared canonical paragraph, preserving the whole
incoming file and appending the owned entry at 350 lines. Preserve all original
issue/doc/corpus bytes, rule/parser bodies, complete diagnostic assertions and
incoming workflow/lock/provider/default behavior. This is independent delivery,
not a claim that the Vapor regression is repaired or an instruction-budget waiver.

TODO: exact new source Actions and necessary existing full Check, fresh healthy
protected projection with all 104 measurements/full suites, actual signed merge
and an included future published version. Published v0.435.0 excludes this PR;
no public-inclusion, native typechecker, whole-CLI gain or 10x credit is claimed.
