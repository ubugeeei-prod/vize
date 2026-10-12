# Sole direct-child setup interpolation in native SSR

Issues: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

The existing native setup SSR entry may admit one original interpolation as
the sole child of one non-void HTML root. The expression still uses the existing
primitive or direct setup Identifier classification. It must retain the actual
original parent, child, AST, File resolution, binding and setup Program owners.

The shared L3 enter/leave collector records the exact admitted child's node,
span and parent in temporary builder state. After the complete walk, it requires
exactly one root, exactly one non-comment root and that same parent root. This
final check prevents a successful prefix from admitting a later sibling element,
comment or void root. The original parent must have exactly the same sole child;
adjacent text, adjacent interpolations and deeper descendants remain refusals.

L4 already spells these genuine reads through Vue's sixth SSR argument and
escapes their runtime values. Its encoder, setup emitter, module assembly and
runtime ABI are unchanged. No second walk, new pipeline stage, L2 operation,
serialization, caller-chosen source pair or legacy dependency is introduced.
Default product routing is unchanged while compiler fix-history
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880) remains open.

This extends the accepted root-only setup SSR scope in
[the original setup decision](./2026-10-04-native-setup-ssr-sfc.md), using the
[original nested receiver](./2026-10-04-l2-native-nested-interpolation.md).
It does not admit broader DOM, Vapor SSR, dialect or component families.

## Preserved before authority

The complete historical fixture files remain byte-exact: the original 15-module,
20-refusal native capture/runtime, their whole source corpus, and the pinned
Vue 3.5.35 / plugin-vue 6.0.7 reference graphs. Capture and runtime refusal index
15 both retain the exact whole source
`<script setup>const value=1</script><template><i>{{value}}</i></template>`
and `Unsupported(Operation)` at node 1, span 49..58.

The actual protected Check
[38186305384](https://github.com/ubugeeei-prod/vize/actions/runs/38186305384)
ran the unchanged whole Rust capture and Node runtime on
`1ffffa6fd66360e418ecb15dad95cd7eb800ad71`. Its successful SSR job
114609715016 uploaded artifact 11719254970. The complete 64,625-byte ZIP digest
is `a98289e072c8d36232abdea0b4fb1bf96f5657989f952d4a7b9be60fd6e7a039`.
Its full native capture and runtime match the committed historical packets.
The relevant native levels, consumer, recipe, helpers and lockfiles are unchanged
between that producer source and this slice's initial base `d3b2f527d86b`.
This authenticates the original named refusal before the change; it does not
claim prior execution of the newly added cases or public package acceptance.

## Finite successor and qualification

Only the named original nested refusal is removed from each current comparison.
The original 20-row archives remain intact; removing index 15 and reinserting
its complete original packet must reconstruct the whole original vector. Every
other capture and runtime field remains under complete equality. All original
nested AST/source/scope/reference custody assertions are retained and now also
required when that source produces an accepted module.

The new corpus contains eight complete original SFC modules: the original named
witness, JS let, TS let, Unicode var, static attributes, TS null, escaped literal
and retained expression comments. Independent authored HTML expectations cover
escaping and mutable setup values. Actual pinned official inline and external
SSR executions are compared with the complete native modules on all three
existing runtime contexts. Recorded/NoLinks bytes, complete raw maps, source
links, annotations, setup ownership and runtime outcomes are retained.

Fifteen additional whole sources exercise adjacent and deeper children,
prefix/suffix/multiple roots, comments, a void suffix and malformed void child,
components, compound/member expressions, unresolved reads and For. Equal-source
foreign setup bindings must remain rejected. Actual `_ctx` and wrong-read
mutations must fail the runtime controls. No map, source, HTML, diagnostic or
unknown field is normalized to select a golden.

The existing native SSR Actions phase runs these laws and keeps both complete
new packets even if a pending freeze fails. Its Rust and runtime statuses must
both succeed for acceptance. The pending fixture is deliberately unqualified:
after one genuine source capture, independently review the full official graphs,
authored HTML, all custody joins and refusals, then freeze the exact complete
new packets and require fresh exact-head Actions. Protected suites and measured
instruction gates must pass before queue admission and actual merge credit.

Remaining TODOs stay explicit: deeper/mixed children, components, For runtime,
Vapor SSR, complete dialect/JSX/TSX routing, and default compiler replacement.
This bounded slice does not close the level or product roadmaps.
