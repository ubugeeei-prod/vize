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

The existing native SSR Actions phase captured both complete new packets in
[Check 38190002082](https://github.com/ubugeeei-prod/vize/actions/runs/38190002082),
job 114620991118, artifact 11720818473, from API source `d00ff674` and physical
checkout `81eeac946`. All 15 relevant source objects match between these heads.
The whole ZIP digest is
`9b48fa7994b76fff27180ca81da095b75823a193d0e7ca557dd337b7c16372e8`.
Independent complete-packet review accepted all eight modules, 15 typed
refusals, 33 native runtime packets, 33 direct executions, authored initial and
updated HTML, raw maps, links and setup joins. All 11 other historical payloads
remain byte-exact; reinserting the one named refusal reconstructs both complete
old vectors. Existing Vite-context differences remain explicitly recorded.

The exact approved freeze digest is
`aa34353f8ed5fc93eb09d20ec466c16aefb9eada57d6d93cc70022a314920ceb`.
Repository Oxfmt changes only JSON presentation, yielding source digest
`3a3ac0f55531e46e72f2abbe07369bbf43c2797bb359dd47bdb91022d689741e`;
the complete parsed objects are equal, including every source and map string.
Its provenance text retains the initial capture's pending-qualification wording.
The original run failed only the deliberately null golden comparisons and is
preserved as failed evidence. Fresh exact-head Actions must accept the frozen
packets; protected suites and measured instruction gates must then pass before
actual merge credit. This source observation does not establish public package
acceptance or complete upstream map/context parity.

The first frozen-source Check
[38191704103](https://github.com/ubugeeei-prod/vize/actions/runs/38191704103)
retained all 15 whole payloads byte-exact, but its final live runtime comparison
exposed a JSON representation boundary. Six exact JS external maps own
undefined `file` and `sourceRoot`; the two exact TS dependency original maps own
undefined `file` and use the actual compiler dependency's public `SourceMap`
prototype. Only the expected side reifies these 14 properties and two prototypes
from the pinned provider. The complete live runtime then requires equal own
keys, descriptors, prototypes and values. Actual unknown undefined/hidden keys,
changed descriptors, missing keys, wrong values/prototypes, altered mappings and
HTML must fail. Every frozen JSON field and raw historical packet is unchanged;
no blanket serialization or output normalization replaces the comparison.

Fresh actual main `47b204f` supplies the already merged lexical native helper;
the original missing-helper and cohort-review failures remain retained. The
three existing release-test roots have complete 36-file local import closures.
Compared with the last installed-Vapor review, only the pure SSR capture-path
matcher changes, to recognize this slice's finite native witness inputs. A fifth
named atomic review at source `78b2d85` records all 36 complete bodies. The four
historical cohorts, excluded roots, commands, oracles and positive native inputs
remain unchanged. The original native-consumer and refusal laws must still pass;
no latest-body allowance or missing-source skip replaces their checks.

Remaining TODOs stay explicit: deeper/mixed children, components, For runtime,
Vapor SSR, complete dialect/JSX/TSX routing, and default compiler replacement.
This bounded slice does not close the level or product roadmaps.
