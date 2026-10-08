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

The paired local preparation decision is [#7882 comment6046345987](https://github.com/ubugeeei-prod/vize/issues/7882#issuecomment-6046345987). Initial read-only pinned stock compilation accepted all eight complete modules; that earlier capture had no local happy-dom and granted no mounted-runtime credit.

Existing read-only dependencies later became available. Independent stock-only
execution of development and production now passes six complete mounts and 18
whole frames against the unchanged literal expected vectors. The first stock
capture exposed an observation alias in the new callback control: its reader
returned the live event array, so unmount retroactively mutated the earlier
snapshot. Preserve that full failed packet; change only this unpublished
authored reader to return a copied array. The original App/Child, old parent
fixtures, expected vectors and generic observer remain byte-exact. Refresh only
the authored source and runtime-pack hashes. The source correction is test-only;
it changes no product operation or expectation. The complete stock descriptors,
script outputs, modules/maps, requests and process observations remain retained.
Vize compilation and paired mounted/runtime/map/performance qualification still
remain unexecuted and require fresh hosted controls after the actual parent.

The existing component-prop walk retains a boolean for an actual static `ref`
attribute or bind argument. Only that existing metadata admits the reused ref
lowering after component creation; components without refs retain their old
operation path without a second ref lookup. `ref_for`/`ref_key` alone and a
computed argument named `ref` do not admit the setter. The existing extractor still scans admitted ref-bearing props and owns their
selection. No allocation or separate stage is introduced; mapped/raw source
and hosted runtime qualification remain pending.

The local child now genuinely descends from the refreshed published parent
`20311a900c3c36e122f0b7faefb2186276dbe951`, after Rust #8195 actually merged
`011d36439050dd0f5fa7113da7152205bd43ea15`. Rebase only regenerated the derived
compiler inventory. The component-ref production delta remains exactly the
reviewed c62 delta; the incoming actual #8133 event-name helper is retained
separately. All authored sources, expected packets and runtime/map helpers are
byte-identical to the reviewed local child, as are all original parent and ref
fixtures. The independently pinned stock-only six mounts and eighteen frames
remain control observations, not Vize qualification. Keep the child unpublished
until this actual parent head qualifies, then publish a native Stack child
without waiting for the parent's actual merge. Current paired Vize mounts/maps,
fresh source Actions and protected original instruction gates remain pending.

The publication decision now permits this prepared child to open as a draft
from parent `20311a9` while its single native artifact-service recovery runs.
The parent's fresh source, whole key runtime, canonical corpus and original
104 performance controls have passed. Its first native body passed, but the
required artifact upload failed and grants no terminal qualification. The draft
starts its own Actions in parallel and retains **UNEXECUTED** status for the
24 paired mounts and 72 frames until hosted execution. Register the actual
parent and child in one native Stack; intake and merge remain root-owned, after
the ready prefix's mandatory exact-head gates pass. This supersedes the earlier
publication hold without changing source, fixtures, expected vectors or gates.

The first draft source check rejected a class-instance spread in the test-only
spawn error packet. Copy its enumerable fields explicitly, retaining message
and stack. An actual ENOENT process packet remains deeply identical, including
code, errno, syscall, path and spawn arguments. Configured type-aware lint and
format pass; original sources, expected vectors and product bytes are unchanged.
The old warning failure remains retained, and the new head needs fresh Actions.
