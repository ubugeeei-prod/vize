# Empty props after template-loop child-key suppression

The exact `6a37d1d27006f75a6b414367513633d5c48cc2df` hosted
`davinci_dom_n8n_default_loops` control rejected the native module at byte686:
the shipped compiler emits `null` after suppressing the sole child key; the
native props writer emits `{  }`. The complete modules both have1013 bytes.

The props writer now emits `null` after its existing visibility filter only
for a plain element whose template-loop child key is suppressed, with no
injected key, scope attribute or surviving prop. Merge arguments keep their
existing path. No retained key binding, span, patch fact, parse, pipeline stage
or public API changes. No instruction or allocation allowance changes.

The named-slot control itself is invalid under stock Vue3.5.26: both official
modes report error33, `<template v-for> key should be placed on the <template>
tag.`, and still return complete code containing the child key. This change
claims Vize's existing legacy byte compatibility for that control; it does
not claim stock acceptance, matching stock output, or runtime correctness.
Both full official modules and exact error locations remain in the receipts.

Eleven complete authored inputs cover pure empty props, bound and static
surviving props, a template-injected key, spread, multiple children, a keyed
grandchild, an actual element loop, and a static child key. Each runs whole
Vize legacy/native module comparisons in Default, Prefixed and Bindings modes.
A separate whole-module comparison keeps the scope attribute in all3modes.
The existing hosted target also retains the unchanged reported named-slot
loop control and original n8n UserSelect input.

Official capture uses the self-contained browser entrypoint
`@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js`, downloaded from
<https://unpkg.com/@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js>,
SHA256 `4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec`.
The capture driver verifies that identity and preserves whole source/template,
options, modules, diagnostics, tips and maps in
`tests/_fixtures/differential/compiler/n8n-empty-suppressed-props/official/`.
Nine controls have stock error33; pure_empty and element_loop_keeps_key have
zero stock errors. These authored sources do not replace any n8n input.

Reproduce with the pinned bundle and an empty output directory:

```sh
node tests/_fixtures/differential/compiler/n8n-empty-suppressed-props/capture-official.mjs downloaded-compiler-sfc.mjs empty-output
```

Cheap local checks cover formatting, assertion policy, receipt identity and
source length. Rust execution remains pending the same existing hosted target:

```sh
cargo test -p vize_l1_to_l2 --profile ci --features legacy-differential --test davinci_dom_n8n_default_loops
```

Fresh entire canonical DOM/reach parity remains required. Historical285 DOM
and233-per-mode reach divergences have not all been classified from capped
snippets, and native workflow success does not qualify those gates.

The fresh `9327e0698eb4049b058e6a50f8611ab621764a13` entire canonical gate
did compare43977 templates with the unchanged16 old-error skips and zero
native refusals, DOM divergences, or production reach divergences. The
inline37345 and module37346 production comparisons were both exact. Those
results qualify that source only; every source correction below needs fresh
entire-corpus proof.

The same source executed all four dedicated targets and rejected the unchanged
`spread_with_key` control: its whole legacy516-byte module uses
`_mergeProps(row.props)`, whereas the native590-byte module uses
`_normalizeProps(_guardReactiveProps(row.props))`. The original source SHA256
is `cf90afb2e428ca911bf73c610213beb0c2b76ae3072bb7a24f65b0f1bb64ff81`.
The complete actual packet SHA256 is
`642367b96168c8b5583a4c6fe4abd954e58cc5da27fa712e37d8d9a845b17dde`;
its native producer binary SHA256 is
`7967a345ff86df756a616ed960953e43ea190235ed9e5c58a8fdb4f01109f885`.
This Default recipe is function mode without identifier prefixing or TS.
Stock error33 for that complete authored control remains unchanged and grants
no successful stock or runtime credit.

Legacy records an authored key in its props scan before omitting that key from
the rendered props. That scan has already selected merge semantics. Native
had discarded the existing suppression flag and reclassified the remaining
argument as a lone spread. The native merge writer now returns that same
precomputed flag with its ordered arguments, retaining `_mergeProps` when an
authored key was actually suppressed. Unsuppressed spreads keep their existing
normalization route. No new scan, parse, stage, allocation, retained IR field,
public API, instruction ceiling, fixture, or expected packet is added or
changed. The existing eleven full controls and scoped control still apply.

The tooling gate also rejected the new `#[path]` test declaration. A move-only
commit relocates its byte-identical module to `tests/empty_props/mod.rs`; the
following change uses ordinary `mod empty_props;`. This preserves test names,
input bytes, relative fixture resolution, and the explicit feature target.
Both bounded corrections remain pending fresh hosted execution, including all
four targets, broad Rust, native, and the complete canonical gates.
