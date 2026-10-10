# Dynamic slot names and local bindings

Tracker: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Parent: [#8359](https://github.com/ubugeeei-prod/vize/pull/8359), exact
`efd843f4ce9387c4782efbc969b5ad9f99b0ee82`.

The current source accepts `#[slot.name]="slot"`, `#[item]="{ item }"`
and `#[key]="{ source: key }"`; the independent Vue rule rejects each because
the slot name is evaluated before the slot's props exist. Diagnose references
to that directive's own local bindings using its existing retained ASTs.
Source property keys, literal strings, initializer reads and enclosing bindings
do not become local declarations.

The shared demanded identifier walk exposes a retained-only entry. It checks
the node's original bytes and preserves lexical scope, comments, computed reads
and incompleteness refusals. There is no fallback parse, full Croquis invocation,
new pipeline stage, source transform or serialized intermediate.

The existing 47-input corpus, all 51 identities, three options, complete source
before/after packets and 188 pinned independent observations remain unchanged.
An explicit eight-entry successor addition manifest names whole new diagnostics
and exact authored UTF-8 spans, including scriptless and LF/CRLF cases. Removing
only these declared additions restores the parent's complete packets and its
original eight-finding/39-unchanged comparison. A separate 16-input corpus checks
identifier/object/array/rest/default binding shapes and negative property-key,
initializer, string and enclosing-scope controls. Both source and independent
provider comparisons retain complete packets and repeat every case.

Local independent provider replay uses the original official Vue base processor
and pinned ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and
TypeScript parser 8.65.0. Only the exact recording filename is normalized; raw
observations are retained before assertions. This preparation needs fresh
exact-head Actions, protected checks, actual signed delivery and public installed
execution. It grants no full51, monorepo, native-product or performance credit.

Initial source Actions at `48df7c6ad` rejected the private `drawer::helpers`
import and its unused internal export. Re-export the helper through the existing
public `drawer` facade and consume that route. The failed build grants no source
or runtime credit; all original fixtures and refusal laws stay unchanged.

The successor source build passes. Its tooling check detects the newly added
Patina helper consumer missing from the generated census. Regenerate through
the existing producer; only the Patina non-product import row changes. Another
shard detects four missing migration-surface rows and the displaced metadata
anchor. Regenerate the former; place the helper module after metadata so the
canonical implementation link and generated global rule indexes stay exact.
The unchanged census and inventory producers pass locally before fresh Actions.

The parser currently cannot retain shorthand-default patterns such as
`{ slot = fallback }` as expression ASTs. Such patterns, malformed argument
headers (including the parent's arrow cases), type-syntax refusals and the
existing modifier-policy differences remain unfinished neighboring work.
They are not reparsed or silently credited here. Dotted static named slots and
every original parent packet remain part of the mandatory controls.

Register this dependent PR with #8359 as a native Stack, keep the emergency
third-party publication hold, then queue only a current-head-green contiguous
prefix. n8n upstream remains strictly read-only and #8142 stays open.
