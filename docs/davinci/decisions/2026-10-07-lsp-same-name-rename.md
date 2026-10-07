# Same-name template bindings during rename

Owning reports: #7994, with reactive-destructure consistency tracked in #7996.

## Decision

Use native symbol selection to discover edits. Parse the retained authored Vue
directives only to recover whole same-name `v-bind` spans and their existing
implicit expressions. A local variable rename expands `:label` to
`:label="title"`; a public prop rename expands it to `:heading="label"`.
Preserve the directive spelling, modifiers and the parser's camelized implicit
value for kebab arguments. Do not discover edits by spelling or sweep other
components. A binding declaration in a reactive destructure selects the local
role even when that one token also names a public prop.

Coalesce complete duplicate edits after expansion. Reject the complete
transaction if native aliases still project conflicting replacement text,
reversed coordinates or overlapping edits onto an authored file. Keep native
scope admission, document versions, annotations and authoritative refusal.
There is no additional native RPC or project source inventory.

## Validation and remaining work

The exact original #7994 report and Child/Parent source bytes are retained in
`tests/_fixtures/differential/lsp/same-name-rename/7994/`. Five Rust laws cover
local/property geometry, native/component directives, modifiers, reactive
binding role, CRLF/astral offsets, truncated aliases, boundary insertions and
plain/annotated conflicts. Four source-built stdio sessions require complete
original local-label/native-id edit vectors, applied whole source, unchanged
Child bytes and empty diagnostics after applying the edits.

At source preparation, Rust formatting and whitespace checks pass. Compilation,
source Actions, protected suites, merge and public package acceptance are
pending. The native prop direction, original multi-component/kebab control,
reactive-destructure declaration/attribute parity (#7996), event/slot/type-alias
linkage (#8011) and non-bundled runtime provenance (#8010) remain separate
qualification obligations. No issue-completion, latency or typecheck speed
claim follows from this preparation. #4075's standard tsgo Content Mapper lane
still requires upstream support; the existing Vize bridge variants remain
covered independently.

## Mixed property and value roles

An actual source-built native stdio control on #8181 head
`dff7005a7a628541cd0922b48ccfb97cc7d6fa5f` confirmed that one transaction's
property role cannot classify every selected shorthand. Renaming Child's public
`id` declaration selected its intrinsic `<input :id>` value and Parent's
`<Child :id>` key together. The global role changed both to `:field="id"`,
leaving a stale Child value and a complete TS2339 diagnostic. A Parent attribute
query additionally followed its implicit value into the unrelated Parent local
declaration. All eight inline/alias, LF/CRLF and public-direction controls
retained these complete failing transactions; four local-direction controls
passed. Applying the complete expected sources in all twelve same-process
controls returned empty diagnostic arrays. These are predecessor receipts,
not qualification of the correction. A separate exact `c86c7a21` main producer
with the same lock, Rust 1.98 and default/native/glyph envelope reproduced the
identical three unsafe inline-declaration edits and TS2339; the complete expected
sources returned empty diagnostics in that same session. Release promotion of
that candidate remains held until corrective delivery and fresh full gates.

Retain each definition-verified `VueComponentPropNavigation` endpoint's mapped
authored argument geometry. A native-selected shorthand changes its public key
only when that occurrence matches a resolved endpoint's URI and exact range;
all other selected occurrences preserve their argument and rename the implicit
value. Prefer the producer's public endpoint for component attribute queries,
and use resolved endpoints for materialized property identities rather than
querying the distinct value identity at the same authored token. This retains
full result vectors without a spelling sweep, DOM-name heuristic or dropping
the selected value edit. Existing scope admission and whole-edit refusal remain.

The original reporter fixtures and four original stdio controls remain exact.
Supplemental inline, alias, interface and withDefaults controls require full
LF/CRLF reference and edit vectors from both public directions and the Parent
local declaration, application of the actual returned edits, unchanged opposite
local roles and empty diagnostics in both files. Fresh exact-head source Actions,
native qualification, protected queue execution, actual merge and release are
required. The remaining #7994/#7996/#8011/#8010 obligations stay open.

Keep the predecessor's empty-position short circuit: ordinary local renames
with no matching component-prop endpoints do not need the reactive-binding
parser/semantic walk. A self-recursive component whose one shorthand occurrence
is both the public property key and that same property's local value needs a
separate dual-role qualification. These Parent/Child controls deliberately keep
the Parent local value independent and do not establish that recursive case;
its full native key/value vectors and post-edit diagnostics remain TODO under
#7994/#7996. No universal shorthand-role or measured speed claim follows.

The corrective `78907bd9` source Check `37606440888` exposed one remaining
Parent-argument failure: the complete public rename response also edited the
independent Parent `const id`. Child-public and Parent-local directions each
passed all eight variants, but the early Parent panic left its other seven
variants unqualified. Broad native phases `37606440151` passed separately.
The public service still supplemented the canonical property transaction with
structural local-value edits. Component attribute queries now return their
already scope-checked canonical answer directly, retaining the final authored
scope guard; ordinary expression, event, model and DOM supplementation keeps its
existing policy. No edit is removed by spelling, and all 24 original complete
reference/edit/source/post-diagnostic assertions remain mandatory on the new
source. Neither those partial passes nor the broad native run qualify delivery.

## Actual Stack delivery

GitHub native Stack #8184 contains #8166 at position 1 and #8181 at position 2.
#8166 actually merged at `2026-10-07T09:21:40Z` as
`8bc19f1c15233fccef229b2fa9044ac62fed8ff2`. Child #8181 remained open after
its queue Check `37595968204` failed because Rust shard 1's completed 4,277-test
results upload stalled and exceeded the unchanged twenty-minute job timeout.
This is observed partial-prefix delivery, not atomic all-or-nothing merging.
The child was refreshed onto actual `main` as `dff7005`, retained Stack position
2, and completed source Check `37600083478` and native phases `37600082576`.
Those green predecessor runs do not qualify the mixed-role correction. Every
layer's actual `mergedAt` and merge commit must be verified independently; Stack
membership, queue admission and green source checks alone are not completion.
