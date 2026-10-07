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
