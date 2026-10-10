# Keep the Vue 2 slot conversion's retained binding role (#8142)

The exact full Check for producer `1e93013d743c52473957b0b897715e8a330970ea`
found a real regression in the original pinned `vue-element-admin`
TransactionTable compiler oracle. Plain `slot-scope` and `scope` attributes
have no expression role. The existing Vue 2 transform created a new `v-slot`
expression without a carrier, while the new slot consumers correctly refused
unretained bindings. Generated callbacks kept their parameter headers but
incorrectly read `_ctx.scope` and `_ctx.row` in the body and filter arguments.

The original source at fixture revision
`6858a9ad67483025f6a9432a926beb9327037be3` has SHA256
`4f32ac35959e8e062b4bfe535523bb5c003ebb9ff783a10eae5f22db869572f3`.
Whole native captures against genuine predecessor
`943702629e503d8b10b4ff9a953c597a268482c0` prove all eight repeated Vue 2/2.7
DOM/SSR packets regress; all four Vue 3 packets are unchanged. The existing
legacy template-sugar test also fails four of ten unchanged cases.

Armature now exposes its existing direct original-source parameter producer
through `parser::retain_slot_parameters_in`. Modern slot parsing selects its
original raw text and delegates to that same producer. The existing capability
gate admits legacy attribute conversion before the newly created slot expression
receives its first compiler parameter parse and retained carrier. Declaration,
default-RHS and diagnostic consumers copy that carrier without another parse.
The guard, complete-tail requirement, single attempt counter, compile arena and
full diagnostic-owner parking remain the same.

There is no attribute field, side table, early destructive normalization or
additional traversal. Public AttributeNode/TextNode/SimpleExpressionNode layouts
stay 56/24/88 bytes on 64-bit targets. Vue 3 and unconverted or valueless attributes do not attempt
the conversion producer. This additive Rust producer export belongs to the
next minor source release. The separate existing Croquis raw legacy attribute
helper remains an explicit boundary; this does not prove whole-pipeline
parse-once.

The original TransactionTable's producer counter changes from five to eight
under Vue 2/2.7 because its three attribute values now receive their counted
compiler admission. Vue 3 producer counters are unchanged. This counter is
distinct from the existing raw reparse probe; no whole-pipeline parser or
allocation improvement is inferred from it.

The focused successor restores all twelve original complete SFC result packets,
including DOM/SSR code, maps, diagnostics, bindings and metadata. The original
legacy-sugar tests pass ten of ten. Additive whole source controls record original,
converted, repeated-converted and transformed roots, compiler results and actual
counter windows before judging. Original typed, entity, guard/error ownership,
default-slot and performance controls remain required and unchanged.

The additive corpus fixes seventeen whole sources before replay and captures
196 observations across Vue 2/2.7/3, legacy/modern spellings and Function/Module
codegen. It checks first admission, repeated-conversion identity, zero attempts
for inert Vue 3/guarded inputs, full retained diagnostics, declaration/default
roles and unchanged ordinary carriers. Modern decoded entity storage already
has distinct raw/content pointers; its original raw and AST pointers remain
unchanged. Only new legacy conversion must share the original attribute value.

The existing full-feature legacy differential action now explicitly runs both
the original template-sugar target and the new handoff target. A normal PR shard
that omits these feature targets cannot qualify them. Its complete new raw
observations are uploaded before qualification, including failed laws. Fresh exact-head source,
full Check, strict 104 ceilings, required native companions, normal review and
protected queue qualification remain required. Earlier 1e93 green slices and
0.440/0.441 installed receipts do not qualify the new source or close #8142.
