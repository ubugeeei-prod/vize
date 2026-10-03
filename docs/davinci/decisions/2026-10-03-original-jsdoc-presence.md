# Original Program JSDoc presence

Issues: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849) and
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The corrected original Vue projection #7547 actually merged at
2026-10-03 12:59:01 UTC as signed 826c032f after exact-source Check
37123027022 and protected full Check 37123673284 succeeded. All 100
instruction ceilings held across three identical measurements. This accepts
that bounded projection; it does not establish complete Vue type semantics.

A subsequent independent official Vue 3.5.35 / vue-tsc 3.3.11 /
TypeScript 6.0.3 oracle checked this exact original source:

```vue
<script setup>/** @type {import('vue').Ref<number>} */ let value=1;</script><template>{{value.toFixed}}</template>
```

Its complete vector contains only TS2322: `Type 'number' is not assignable to
type 'Ref<number, number>'.` at original SFC line 1, column 60. Actual native
TS 7.0.2 checks the unchanged JS setup and original member read in the same
configured process. It retains that TS2322 at script line 0, UTF-16 45..50,
but additionally reports false TS2339 at projected line 4, UTF-16 6..13:
`Property 'toFixed' does not exist on type 'Ref<number, number>'.`
Vue unwraps the declared Ref type; primitive initializer eligibility does not
certify the effective type carried by the original JSDoc. Neither diagnostic
is suppressed, normalized away or replaced with a fabricated context.

The prerequisite adds one private presence receipt where the original stock
lexer already classifies `CommentContent::Jsdoc` and `JsdocLegal`.
`TriviaBuilder` updates that fact in its existing comment insertion and
lookahead-dedup path. Both existing checkpoint variants retain and restore it;
the existing committed Unambiguous top-level-await reparse retains the
completed tail's fact. The independent original legacy-literal receipt remains
unchanged. The ordinary original parser return retains both facts privately.
Only a clean original `AdmittedProgram` exposes immutable JSDoc presence.
This is presence, not an effective type, comment-to-binding association,
semantic admission or a guarantee that all TypeScript JSDoc forms are covered.

There is no extra parser invocation, pipeline stage, comment/source scan,
AST walk, copied comment vector, caller-supplied boolean or mutable authority.
Original Program, comments, requested JS/TS profile, bytes and diagnostics
remain the normally owned observation; failed syntax retains its comments and
full diagnostics without minting admission.

Focused original laws cover JS/TS positive and ordinary-comment/string/regex
negative cases, moved owners, same AST/comment pointers, original spans,
lookahead, both checkpoint variants, deduplicated re-consumption, committed
await reparsing and syntax holes. A native L1 embed law retains the same
Program/source identity and absolute authored comment mapping after moving
its owner. Validation runs in source-built Actions; no local Cargo build or
backend installation is needed for this prerequisite.

The next genuine child must consume this receipt in L4's existing first
setup-binding read check: JS setup with JSDoc refuses template binding reads
until faithful Vue type unwrapping exists. Full script-only and literal-template
copies remain admitted, ordinary comments remain admitted, and TS source
retains its original language-specific path. Existing runtime primitive and
Read-usage proofs remain required. The configured Canon consumer #7554 stays
drafted and unqueued while this prerequisite and child are incomplete.
Source checks, protected full checks, immutable all-100 ceilings and actual
merge are separate acceptance requirements; new admissions wait behind the
shared census fix #7580. Default replacement and #6849/#6879 remain unfinished.
