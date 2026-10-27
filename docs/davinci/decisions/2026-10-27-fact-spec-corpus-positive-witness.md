# Preserve the original fact corpus and add an independent positive witness

Checking same-element `v-for` collection expressions as enclosing-scope reads
removes the last `unused-bindings` rows from the original tooling corpus. A
collection used only in `v-for="placement in placements"` is read; its former
unused row cannot serve as evidence for the unused relation.

[Check run 40486958144, job 122131738977](https://github.com/ubugeeei-prod/vize/actions/runs/40486958144/job/122131738977)
compared all 773 original SFCs at PR #10933's merge checkout
`1343403f3f7628f0d9c8d20b0ac1513fc1277d19`. Production and the independent spec
agreed on zero unused rows, and the zero-fact refusal correctly failed the gate.
The original pins and census are:

| Project        | Gitlink                                    | SFCs |
| -------------- | ------------------------------------------ | ---- |
| ant-design-vue | `7483836f0adac76516e527df893c3d84a7cd4005` | 731  |
| create-vue     | `3ee62adffdcdfa4a37b2ed4e9c30636655d5fcd1` | 42   |

Keep all those comparisons and the unchanged `Agreement::verdict` law. Add the
already pinned [layoutit-grid corpus](https://github.com/layoutit/layoutit-grid/tree/bac9964b1d23a361f3ef9e67b0fdd2d990b48a1a)
(`bac9964b1d23a361f3ef9e67b0fdd2d990b48a1a`, 112 SFCs) as a separate positive
witness. This project's
[`CssCodeGridArea.vue`](https://github.com/layoutit/layoutit-grid/blob/bac9964b1d23a361f3ef9e67b0fdd2d990b48a1a/src/components/code/CssCodeGridArea.vue)
contains an unread setup declaration, `includeTemplateAreas`. Its authored
source is analyzed without edits. The upstream
[`LICENSE.txt`](https://github.com/layoutit/layoutit-grid/blob/bac9964b1d23a361f3ef9e67b0fdd2d990b48a1a/LICENSE.txt)
is MIT, copyright 2017-2020 Leniolabs; the submodule preserves it.

The existing [Davinci unused-bindings run 40486958031](https://github.com/ubugeeei-prod/vize/actions/runs/40486958031)
passed at `fb6a52ed0e3f6dd737e375c57cb991232d6f565a` with this exact project pin
alongside splitpanes and cssgridgenerator. That run proves the existing complete
three-project lane passed; it does not by itself give layoutit-grid's individual
fact census. The new separate witness scope provides that census.

`VIZE_DAVINCI_FACT_CORPUS` keeps the original selection.
`VIZE_DAVINCI_FACT_CORPUS_WITNESS` optionally supplies additional real projects.
The harness prints `corpus originals`, `corpus witnesses`, and their summed
`corpus shard` scopes. The witness must independently compare positive unused
facts, and the combined shard retains every exact-table comparison and every
existing verdict. The tooling gate pins 773 original and 112 witness comparisons
and checks the summed counts. Positive combined rows are not attributed to the
original corpus. Other callers that supply no witness keep their original
zero-fact refusal.
