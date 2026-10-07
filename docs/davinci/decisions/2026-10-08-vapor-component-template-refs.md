# Component template-ref ownership for #7882

This local slice prepares the remaining component-ref clause as an unpublished
child of #8101. The original App and Child remain 380 and 104 bytes, with their
existing manifest hashes unchanged. The original complete result must be
`world / 3`; the delivered native-element loop-ref clause alone never qualified
that result. All old key, loop-ref, source, runtime-observer and map fixtures
remain unchanged.

The existing component-prop walk excludes static `ref`, `ref_for` and `ref_key`
metadata. Immediately after the existing CreateComponent operation, reuse the
existing template-ref lowering with its existing component ID and lexical loop
depth. It uses the public captured-owner setter, including queued assignment
and cleanup. The independently pinned [Vue runtime contract](https://github.com/vuejs/core/blob/5be279570a844b953dd44b56bdc2426f7421aecd/packages/runtime-vapor/src/apiTemplateRef.ts)
accepts the component handle and obtains `defineExpose` through `getExposed`.
No private instance provider, new operation, parser, pipeline stage, extra
level serialization or DOM substitute is introduced.

The retained argument must be static before it is treated as `ref` metadata.
An authored computed argument named `ref` remains an ordinary computed prop;
the independent callback control combines it with the genuine static `:ref`
directive. This also preserves the actual component callback and null cleanup.
The old native-element input and observer bytes stay unchanged.

Three complete runtime controls prepare four actual compiler recipes: both
development/production modes and inline/separate output. Each uses the unchanged
generic SFC runtime observer and a separate pinned Vue compiler producer. The
stock reference always uses inline output at the matching production flag; it
is a semantic runtime oracle, with no stock/actual byte-output parity claim.
The original complete App/Child, a component loop followed by a scalar
conditional sibling, and computed-prop/callback source retain every node,
namespace, event and exposed-value field. Removal, clear, hide and unmount
preserve complete vectors. The planned total is 24 actual/stock mounts and 72
whole frames. Full map-on/off public SFC Results, exact source contents,
stock modules/maps/script results, requests and process output are saved before
runtime assertions. Errors and failed processes grant no successful credit.

Hosted execution remains **UNEXECUTED**. The child stays unpublished until Rust
actually merges and the genuine #8101 refresh is qualified; then register the
actual parent and child in a native Stack. Require fresh source/native gates,
the unchanged full canonical corpus and original instruction ceilings before
root intake and actual signed delivery. Existing broader direct-Ref-expression
coverage, native-only compiler migration, reporter rc.10 execution, SSR,
hydration, browsers and installed-release acceptance remain unfinished.

The paired local preparation decision is [#7882 comment6046345987](https://github.com/ubugeeei-prod/vize/issues/7882#issuecomment-6046345987). Read-only pinned stock compilation accepts all eight complete modules (three inputs plus Child, development and production); their full descriptors, script outputs, modules and maps are retained locally. Mounted stock/current runtime remains unexecuted because the local read-only dependencies lack happy-dom; this does not change the hosted recipe or grant runtime credit.

The existing component-prop walk retains a boolean for an actual static `ref`
attribute or bind argument. Only that existing metadata admits the reused ref
lowering after component creation; components without refs retain their old
operation path without a second ref lookup. `ref_for`/`ref_key` alone and a
computed argument named `ref` do not admit the setter. No allocation, new walk
or separate stage is introduced; mapped/raw source and hosted runtime
qualification remain pending.
