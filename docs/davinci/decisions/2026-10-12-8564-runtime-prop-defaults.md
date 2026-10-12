# Compiler-owned runtime prop defaults

Refs [#8564](https://github.com/ubugeeei-prod/vize/issues/8564) and
[#8090](https://github.com/ubugeeei-prod/vize/issues/8090).

## Decision

A direct runtime `defineProps({ value: { required: true, default: 1 } })`
in an inline, sole `<script setup>` block must report the existing
Error/no-fix `script/no-required-prop-with-default` diagnostic. Previously the
rule's prefilter admitted these calls, but neither its typed nor Options
producer inspected the runtime object.

Keep the existing typed producer byte-exact. Preserve Options API primary
key spans, all existing messages, labels, help and severity. The new runtime
producer uses the complete property declaration as its primary span, as
independently established by the pinned
[eslint-plugin-vue 10.9.2 rule](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/rules/no-required-prop-with-default.js).
Its two native labels still span `required` through the literal `true` and
`default` through the complete value expression. No fix is invented.

The production change is restricted to the actual configured native rule.
Its default trait capability refuses configured namesakes, which keep their
existing SFC dispatch. Normal scripts, standalone JS/TS and inline HTML keep
the previous rule dispatch. The setup producer consumes the existing shared
OXC parse and adds no parse, sibling request, Canon API or pipeline stage.

The compiler-macro binding guard is moved into a shared helper in a separate
move-only commit before it is parameterized by macro name. The existing
`defineEmits` caller supplies its unchanged name. A runtime object checks
ownership lazily once per block; typed calls do not repeat that binding scan.
The existing namespace, import, hoisted-var and destructuring refusals remain
conservative. Genuine `vue` imports of the matching compiler macro are allowed;
foreign/default/namespace/non-matching imports establish a binding instead.

Only direct expression statements and non-ambient variable initializers are
examined, after the existing parentheses/TypeScript wrapper unwrapping. A
non-optional bare `defineProps` call must have no type arguments and a literal
runtime object. Props with a literal `required: true` and a `default` member
are reported. Aliases, nested calls, member calls, bound objects and mutable
resolution are excluded. A normal sibling or external `src` refuses this
new ownership claim regardless of whether its body is filtered out.

## Frozen evidence and scope

The original usable finding class comprises sixteen runtime findings in
seven whole Hoppscotch sources at
`hoppscotch/hoppscotch@1acb8a3a7581e4db32ba0d529170c4669a2e1053`.
The complete original Matrix shard packet remains unchanged. Hoppscotch's
broader whole-project packet contains an unrelated provider parse failure;
this change cannot qualify that project or the complete Matrix.

The seven source bodies are retained without edits. Eleven independently
authored controls add three genuine positives and eight negatives, including
local/foreign bindings, sibling ownership and external setup refusal. The
pinned official provider has twenty-three target findings across the eighteen
sources; the native contract has nineteen. The four differences are explicitly
named conservative ownership refusals rather than silently changed provider
expectations. A separate original foreign configuration message is also
retained in the raw provider packet.

Primary qualification uses one frozen, explicit Incremental configuration
containing only this rule at Error, plus the twelve separately frozen ownership
controls. It compares the entire resulting API and
CLI packet, raw process output/status, messages and ranges for all eighteen
sources. It does not project a whole default-project result after execution.
Seven minimal sources additionally qualify complete default/recommended,
Essential, Ecosystem, Incremental, explicit-off and n8n configuration routes,
including their complete companion diagnostics and order.

The provisional target-only seven-route arrays were authored before
execution, then found to misstate explicit rule admission for Incremental.
They are retained as unexecuted preparation evidence. Both native
`with_enabled_rules(Some(...))` and CLI explicit rule configuration admit the
named rule independently of preset membership. The corrected scoped contract
uses that actual builder behavior; the separate minimal full-route contract
continues to test membership without an invented global disable mask.

The authenticated old public v0.443.0 before packet is retained for all
eighteen sources. It is historical evidence, not a fresh public qualification.
All original392, setup357 and reserved443 vectors and their original
206 distinct fixture authorities remain unchanged. No budget, historical
failure, source pin or existing oracle is relaxed.

## Exact first-source custody

Initial source `029b7facb57d56e0c94780a4bc1f0820ceef996b` builds successfully
on [Check38183136962](https://github.com/ubugeeei-prod/vize/actions/runs/38183136962).
Its complete79 CLI packets pass unchanged, with compiled source
`1b20ab16662301832717cbfd072d9e1bfe9c12ff` and native binary digest
`4e83d4ac1889f5840aa6010ed793bcffc8f579fd11dffff7ebd8efaa889acaea`.
The raw79 packets and both complete failing tooling logs are retained under
`source-custody/`, with compressed/raw hashes in `initial-qualification.json`.
This grants no overall source-green or successor qualification.

The original reserved API order law correctly refuses the changed whole
`script_rules.rs` body; its sibling carrier would also differ. Its immutable
prior custody fixture, original full law and all frozen vectors remain readable
and unchanged. A finite successor admits only those two carrier paths, requires
the complete twelve-file runtime production family, authenticates every whole
before/after archive and preserves the exact original rule iterator and ordinary
AST dispatch body. All other original source pins keep their original check.
Every source/archive/authority mutation and mixed family refuses admission.

The five added source-comment lines shift the rule implementation location;
the rule-reference generator correctly refuses the stale implementation links.
Regenerate only those links through the eleven affected reference/index outputs,
retaining all examples, aliases and other rule bodies. No generator or reference
oracle is relaxed. Fresh repaired exact-head Actions remain required.

## Remaining work

- The twelve supplemental ownership/syntax controls are frozen before execution.
  Their separate provider qualification remains pending; they add one native
  positive and eleven meaningful negatives. Source-only custody passes without
  skips. Source API and CLI each retain all79 packets:18+12+49.
- Run complete source API/CLI controls and preserved old corpora on exact-head
  Actions with the existing shared CLI build receipt.
- Obtain mandatory native phases and the five protected workflows; follow the
  independent PR through actual merge.
- Obtain a fresh successful official release containing the actual merge and
  qualify the installed CLI. Root owns official public dispatch.
- Full default-project parity, current whole Matrix, typed/Options primary
  range disagreements and the broader #8090 requirements remain unfinished.
- Type-only namespaces conservatively refuse runtime macro ownership. This
  named false negative is retained; the existing emits limitation is unchanged.
