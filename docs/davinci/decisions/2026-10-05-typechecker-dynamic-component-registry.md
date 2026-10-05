# Dynamic component registry checks (#8003)

The original [P0 report #8003](https://github.com/ubugeeei-prod/vize/issues/8003)
shows CLI and editor TS2339 on `__vize_dynamic_is_N` for the built-in
`<component :is="'button'">` whenever `checkUnknownComponents` is enabled.
Its full issue bytes are retained locally; original body SHA-256 is
`987008b0a555afa2ede3d82ad418c54c08ea06d22e2cb3f509842c3085c7276b`.
Reporter `ubugeeei` / GitHub ID71201308 is verified from the live issue API.
The independent source starts at actual main
`66a9b15639c828b17d5f655b3dd58f697799b3fa`; it does not modify the separate
configured Batch-session/publication repair, helper materialization, releases,
old CPU campaign or its source/driver/raw profiles.

Croquis gives non-identifier dynamic component expressions an inference alias.
The existing template scope declares that alias and checks its props/events.
`GlobalComponentPlan::emit` incorrectly checks the same synthetic name against
Vue's global registry and emits a second module-scope fallback declaration.
The repair excludes only an alias proved to belong to its actual authored
`<component>` element with a bound `is` expression. Proof combines the exact
Croquis alias derived from the usage offset and the existing AST locator;
matching a name prefix alone is insufficient. Valid authored unknown component
tags retain registry checks. A parsed ordinary element with an adversarial
exact-alias/offset fact cannot prove generated ownership; the AST locator also
declines `:is` on an ordinary authored component tag.
The private plan constructor receives the matched facts and AST together;
the existing generator layout and oversized-file line count stay unchanged.

This changes the existing generator, with no new product stage, checker API,
helper declaration, ambient global, diagnostic filter or `any` fallback.
Dynamic value/name resolution, constructor inference, props checks and source
mapping stay in their existing paths. Complete generated code and projection
mapping are compared with the default option for literals, conditionals,
lookups, calls, refs/spreads, independent same-element loops, unknown expression
reads and constructor-union props. Static unknown tags keep authored registry
mappings across Unicode/CRLF; a valid unknown tag containing a reserved-looking
name remains a real check.

The legacy fixture retains the original App.vue, tsconfig and editor config.
The existing runtime tooling suite exercises the real CLI and stdio LSP with
explicit true, false and absent `checkUnknownComponents`; each live session
moves through original/expanded dynamic cases, a real missing component,
an authored internal-looking tag, an unresolved dynamic expression, a bad
known-component prop and repair. Complete CLI and diagnostic publication
objects are retained before assertions. Expected counts, codes, order, authored
starts, severity and editor source are checked; clean cases require empty
complete arrays. No new normalization or message-deletion path is added.
The existing always-upload corpus action retains these objects under
`target/differential/typechecker-dynamic-component-8003/receipt.json`.
Existing full differential and native-runtime gates remain required.

Validation is source preparation only: rustfmt parsing, configured TypeScript
format/lint and pure inventory checks run locally. No local native/Rust build,
provider installation or old-binary outcome qualifies this change. Fresh
exact-source Actions must prove generator laws and the real native 7.0.2
CLI/editor runtime cases. Ordinary, full-source and protected queue results are
reported separately; a failing candidate is removed. Actual signed merge and
published product release are still pending. #8003 stays open until the named
member control below is also covered and the complete fix actually merges.
Every instruction ceiling remains unchanged.

The same decision is recorded on #8003 and in the canonical record in this
slice. Include the verified reporter as a Co-authored-by trailer. Independent
technical peer review qualifies only the frozen source; it does not transfer
historical CPU, runtime, merge or release acceptance.

The first hosted source `a5988e0402e42d7b3d9a3e8e963313588a076f16`
failed before native execution: the new usage type named the crate root rather
than the existing public `vize_croquis::croquis::ComponentUsage`; the scope
re-export required sorting, and repository formatting rejected original App.vue.
The successor fixes the public path/sort and adds only this exact fixture folder
to the existing formatter-sensitive source policy. All original fixture bytes
and all 21 assertions remain unchanged. The raw logs and failed source identity
are retained; they prove no native/unit result. Full-source dispatch waits for
the separate #7857 `stable_revision` native-feature repair to actually merge,
then requires a fresh actual-main rebase and all hosted runtime checks.

Hosted `fed0c4880f5d217a839cb4e964004efe920a2caa` fixed those initial
compile/format failures, but ordinary Check37308329258 still failed. All five
new generator laws failed: four reached the bare parser helper's non-void HTML
self-closing recovery, and one incorrectly expected an underscore-start tag to
be a component. Worker2 alone did not qualify the other three laws; subsequent
worker3/4 logs establish their failures too. The existing tokenizer accepts
only ASCII letters at a tag start and retains those underscore strings as
opaque text with no component usage. NativePhase37308327991 separately passed
its default suite, without executing the new 21-state CLI/editor control.

The narrow successor uses the production DOM native-tag callback in the mapped
unit helper, so actual component tags use the same classification as production.
The runtime reserved-looking positive is `Unknown__vize_dynamic_is_777`, keeping
all 21 states, complete diagnostic vectors and original three fixture bytes.
An actually parsed `Unknown` with and without bound `is` supplies an explicit
low-level exact-alias(0) fact-mismatch ownership refusal. The three original
underscore-tag strings remain honest no-usage, complete code/map controls;
no false TS2339 expectation is replaced with an empty positive. Croquis's
existing generator also refreshes its stale observational 20-file inventory,
which caused the two tooling1 failures. Production alias eligibility, options,
offsets and bound-is behavior remain unchanged. Raw failed logs are retained;
new source peer and hosted execution are required, with no inherited pass,
performance or release credit and the separate actual-main full-check hold.

Before publishing that grammar successor, static review found a second test
assumption hidden by the earlier parser failure: `registry.button` is a known
static-member target under Croquis's existing authority, not a generated alias.
Its unconditional alias-presence assertion was unjustified. The bounded alias
law uses `registry['button']`, the computed-member form that Croquis actually
aliases, retaining complete code/map equality and the same authored object.
No named-target binding policy or production eligibility is broadened. The
possible separate static-member registry behavior remains unmeasured and is
not claimed fixed by this synthetic-alias repair. The uncommitted 0eddf9 review
is retained only as a historical grammar/hash audit; its generator-law source
clearance is withdrawn and a new exact-source peer/hosted result is required.

The original named-member input remains an explicit pending native control:

```json
{
  "script": "const registry = { button: 'button' };",
  "template": "<component :is=\"registry.button\" />"
}
```

It must be clean with explicit true/false/absent unknown-component options;
wrong-member/value/known-prop errors must still be retained. This exact case
has no hosted result for the correction. The computed-only successor may be
published as Draft source preparation, with the named-member gap stated in
the PR and its automatic #8003 closing reference removed. It grants no Ready,
queue, P0 closure or runtime credit; the complete AST-owned repair remains open.

The next private source slice adds only named-value registry eligibility. The
existing offset locator returns the original bound-is expression node; a named
owner requires its retained AST to be an Identifier or StaticMemberExpression,
its retained raw bytes to equal current node content, and its trimmed name to
match the actual Croquis usage. An authored ordinary tag, offset/name mismatch,
stale parse or missing AST does not qualify. No dot/prefix exemption or new
parse, pipeline stage, global, `any` fallback or diagnostic filtering is added.
Only registry diagnostic destructuring and its associated mapping are omitted;
all original named declarations, value checks, bindings and prop paths remain.
A complete module-byte/map law checks that exact boundary against declined AST
ownership, alongside fresh/stale/no-AST/ordinary-tag refusals.

The original 21 CLI/editor states remain intact. Four additional states per
option preserve the exact named-member input, wrong member, invalid numeric
member method and bad known namespace prop, giving 33 prepared complete native
states. Raw observations are written before strict assertions; no failure may
be normalized or downgraded. Namespace props may expose the pre-existing
sanitized binding gap: only authentic failed control evidence can justify a
separately reviewed typed-path follow-on. Until every real error and complete
runtime gate passes, #8003 remains open and PR8040 remains Draft/unqueued.
No local native execution, actual merge, performance or release credit is
claimed. Fresh peer/source Actions and repaired actual-main full execution are
required; the finite first433 publication hold also blocks queue admission.

Published grammar source `3c78cc2bc5e49be24a399f9e486338e2c7f3f74f` has terminal
PR Check `37316229933` success at actual virtual checkout
`d71e2cda250dc90aa281886db621c5ec35ac725c`. Its four affected workers ran
16,161 tests successfully, including all five original generator laws; this
is affected-source evidence, not the full protected or 33-state native gate.
NativePhase `37316225942` failed before compilation: its newer merged workflow
requested `check_vue_helper_scope_cli`, absent in the original source head.
The complete 219,173-byte failed log has SHA256
`b9113b67cde64a0048164674fdd44df85935bc2972cc1c41a1dd435ed605a4d3`;
official artifact `11347453747` retains only source custody, 2,674 bytes with
SHA256 `5d383e5857edbd43a1be784bceac3912a46903edb1673540539f5fc1e56e0398`.
The observed workflow SHA is `d71e2cda250dc90aa281886db621c5ec35ac725c`,
source/driver remains `3c78`, baseline `7c591c8e8f0af608e67746c592e40af3d1b6031f`,
and fetched main was `d8cd6a208b9aea02150b140a4b81b87222128a51`. No native
component state or phase was executed; no missing test is waived or removed.

After actual signed #7857 merge, the existing three commits and private named
owner slice are genuinely replayed onto `92e036de6f8dc651440e7c9a2f415a3c7fa69890`.
All incoming helper/session/publication/workflow and other decision clauses are
preserved; the same original three fixture bytes and source-only 33 states
remain mandatory. The source/workflow mismatch requires this fresh-main
composition, a new independent exact-source review, source/native Actions and
full native runtime execution. Prior passed affected laws and failed native
receipts stay historical; the first433 hold, open Issue and Draft status remain.

Exact named-owner source `19d2d4fe0884392a0484325fcae5e41683401a73`
passes PR Check `37320325580` at virtual checkout `5672c7dd3de406371a3ccd2d634a4d24f41124e3`;
its tree equals source tree `98b9986c28ea06efe9723baee9dbee098b3120ee`.
All four affected Rust workers pass 16,179 tests, including the seven generator
laws. NativePhase `37320324574` also succeeds with `profile:none`, retaining
official artifact `11349688661` (81,698,833 bytes, SHA256
`402ca4c1f8fc1ac94374ff7f483e8c38525137b0bd58f42ed61d494bd1132d6b`).
These are source/default observation gates, not the 33-state or protected gate.

The required full Check `37322063497`, source/driver `19d`, is terminal FAILURE:
only `test-scripts` fails, while source coverage, full Rust and Vue parity pass.
The unchanged native control emits no CLI or LSP diagnostics for
`<component :is="registry.Choice" count="wrong" />`, where `Choice` requires a
numeric `count`; its independent expected TS2322 remains at line 6, character 43.
The complete failed log is 1,814,638 bytes, SHA256
`7e8a125da7575d6d01021de9c60169f77344a91951c5a57b54b1f553eb2e448f`.
Official artifact `11351119302` is 58,655,096 bytes, SHA256
`ed1ae8a1f1e84692f0939e84727ee9779d228cae1291f80aa0f0e5059eb7a2fb`.
Its original dynamic-component receipt has 11 true-option observations: the
first ten pass and the namespace-prop state fails; false/absent and the remaining
22 states are unexecuted. The source-built binary has SHA256
`e7cf038e4259e6f0f35d6a411502a840a42f27635291cfd842d7854682cbd6dd`.
No 33-state success, complete Issue fix or release credit is granted.

This actual failure authorizes a separate private typed-path follow-on. The
props declaration currently sanitizes `registry.Choice` to `registry_Choice`,
losing the constructor type; default props eligibility can also omit it.
Only those two props boundaries gain the existing, fresh offset/name-owned
bound-is AST reference. The Identifier root resolves by its exact script/import
or external value binding, preserving Props aliases and rejecting type-only
names; markup Pascal/camel candidates cannot replace that root. Static member
properties use retained decoded AST identifiers; unsupported roots decline.
No per-query parse, global, `any` fallback, provider change or new stage is added.
Independent source review finds that the new OR would broaden legacy Vue 2
props eligibility and exact-root reference choice. The new private reference
therefore declines legacy Vue 2 and plain Identifier names at entry, preserving
both unrelated old boundaries. The negative-only dot gate avoids the existing
whole-template locator for ordinary plain names; it grants no ownership and
every admitted member still requires the fresh AST proof. Recursive exact
Identifier root resolution applies only inside members. No measured cost or
performance claim is granted. A
whole generated-code/map law compares actual fresh named-value ASTs against
the same source/summary with the new-reference route declined by missing
retained AST, including a lowercase/uppercase binding collision in both modern plain-name
and legacy modes. This is
unexecuted source coverage, not a native Vue 2 behavior claim. Ordinary tags
retain their old markup reference. Mapped laws require exact count/value sub-spans and generated check/literal
bytes (broad parent-range containment is insufficient), and cover namespace,
case collision, nested/supported escaped roots, Props aliases and exact
import/external/type-only authority. All original fixtures and 33 native states
remain unchanged; fresh independent peer, exact source/native/full Actions,
protected ceilings, actual signed merge and release remain required. This
private follow-on has no Rust/native execution or measured performance claim.
#8003 stays open, PR8040 stays Draft/unqueued and the first433 hold persists.

Before freezing the typed follow-on, all four existing source commits are
replayed cleanly onto actual main `8a8521d6897bbe3fd0af0cbfaebd83f4fc933933`
(the first433 release commit). Incoming version/lock/workflow/decision changes
remain intact. The original `19d` failed binary and receipts stay historical;
new source/native/full gates must use the newly published source. The refreshed
consumer inventory labels the new mapped unit module as test/dev. No release
publication or Ready/queue authorization is inferred from the main advance.

Static producer trace also corrects the new escaped-root positive premise:
lowercase raw `regis\u0074ry` is not the decoded `registry` binding and does not
pass Croquis's existing uppercase-or-binding target selection. The positive
uses actual supported uppercase `Regis\u0074ry.Choice`/`Registry`, proving the
retained AST's decoded reference. The lower escaped input is retained as a real
no-usage, complete-output/map unchanged control. No ComponentUsage is fabricated,
Croquis widened or original fixture changed; this is an unexecuted source
assumption correction, not a reported native failure or success.

The same source trace finds that `(registry).Choice` is also not admitted:
Croquis's raw root is `(registry)`, not the declared binding, and starts with
`(` rather than an uppercase letter. Its original bytes therefore move from
the new positive table to the same actual no-usage whole-output/map control.
The projector's parenthesized recursion remains defensive without actual
producer/native coverage credit. All remaining positive premises are traced
through the real root-binding/uppercase selector and implicit Props producer.

A final static API check uses ProjectionMapping's actual public `spans()`
accessor in the new mapped law; the initial private `.iter()` premise is
corrected before hosted compilation. Expected spans/bytes, original controls
and all production blobs remain unchanged; no failed runtime is inferred.

Published typed source `a0e8c3de6de160f2fa965521d2fe68daec048409`
(tree `6cfebe9dc361fe28ab6dd5425bcda5b5d15c0b34`) has terminal FAILURE in
Check `37332747295` and NativePhase `37332746363`. Both expose three E0502
borrow errors in the new unit closures: inferred parameter lifetimes retain
the type-only set across two insertions and the Root across its later AST
mutation. No new generator unit or 33-state success is granted. The affected
build log is 209,816 bytes, SHA256
`992c2e1667b26444c3dcd0e122df25166b4357612b4f0e9c6f5c882e87736363`;
the native job log is 295,073 bytes, SHA256
`351d9aff7235fe9115a06f6b98b0d598f5cf1aa9de44f70101ef16a4a05a11cf`.
Its official partial artifact `11355440909` is 3,729,626 bytes, SHA256
`d648781a4caf4bad6619a986a22a5b7dd1eed69737bb1b65befb11b2ef8f8815`,
with 671 distinct archive members and complete ZIP CRC verification; unrelated
partial control reports are not the dynamic 33-state or whole-phase gate.
A test-only successor annotates both closures' borrowed input types explicitly.
All production blobs, expected vectors/subspans/bytes, original fixtures and
33 runtime states remain exact. Fresh peer/source/native/full Actions remain
required; no lint suppression, provider change, local native execution or
result transfer is introduced. Draft, open Issue and admission hold persist.
