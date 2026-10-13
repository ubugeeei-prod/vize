# Iteration key rule ownership

[Tracking issue #6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The pinned v0.448.0 real-project matrix exposed 40 extra
`vue/require-v-for-key` diagnostics and 25 missing `vue/valid-v-for`
diagnostics across seven projects. Twenty-four extra findings come from custom
components or component-only transparent templates. Two extra findings come
from transparent template loops around empty slot outlets. The remaining fourteen are
in a file whose original `eslint-disable` comment requires separate suppression
handling; valid unkeyed native elements must continue to be reported.

The [pinned upstream rule](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/rules/valid-v-for.js)
assigns missing custom-component keys to `valid-v-for`.
`require-v-for-key` keeps ownership of native repeated elements. Custom tags,
including lowercase registered components and native tags with `is` bindings,
are checked by `valid-v-for`. An unkeyed template forwards that validation to
its children. A nested loop consuming the parent's iteration variable is
checked by its own loop, while independent child loops retain both checks.
Iteration references use parsed identifier references, so string literals and
static property names cannot silently satisfy the key requirement.

The existing #8281 slot/static/object-key exemptions and historical any-keyed-
child policy remain unchanged. Its original fixtures and all 21 captured before
API packets remain intact. A separate complete 21-packet after oracle changes
only the empty-slot expectation to zero: there is no native repeated leaf that
requires a key. Unkeyed native fallback leaves still produce their original
positive diagnostic, and custom leaves remain owned by `valid-v-for`. JSX mapped components still require keys, and petite-vue keeps
its optional-key behavior. The new 24-case focused corpus
covers native positives, custom component negatives, nested loops, SVG,
literals, dynamic components, and CRLF/Unicode opening-tag ranges.

The comparison map and zero false-positive/false-negative budgets are unchanged.
Actions source checks, protected queue qualification, actual merge, and the next
published consumer replay remain required; this source change alone proves
neither release readiness nor full matrix completion.

[Actions at 8524a9a](https://github.com/ubugeeei-prod/vize/actions/runs/38335119019)
confirms the first 22 template vectors pass. The two actual Habitica empty-slot
false positives require correcting the old current expectation. Its original
before packet remains preserved, with the new after packet checked separately. A JSX positive initially used
`const view = items.map((item) => <Row item={item} />);`; that standalone call is
outside the current render-root collector and produced zero diagnostics. The
supported JSX render-function positive keeps an expected missing-key diagnostic.
The bare-call collection gap remains unfinished under #6830; this change does
not broaden the render-root collector or claim that standalone form is covered.

The generated import inventory gains eight owned rows after normal helper
module extraction. Its existing source-custody
manifest updates only that one current-body identity, retaining all 322 other
current bodies and every historical frame, law, family, and credit field. The
helper and its existing test retain their inverse-checked whole-body pins. This
bookkeeping grants no native or performance credit and changes no predicate.

Helpers and syntax tests move into ordinary Rust modules without changing their
behavior, keeping each source below the unchanged 350-line ratchet. Generated
rule references and their materialized catalogue dependencies receive the
canonical implementation-link updates. The 24-vector and 21-slot tests retain
complete actual API packets in the existing Rust shard artifact directory
before assertions, including failure packets; no workflow stage is added.
