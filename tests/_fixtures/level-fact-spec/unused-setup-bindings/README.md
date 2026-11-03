# Unused setup binding regression

`LoopSource.vue.txt` distinguishes a collection read by a same-element `v-for`
from bindings nothing reads: `colors` is used, while `unusedLabel` and
`unusedCount` remain unused.

The fact-spec test reads this complete source with `include_str!`; the `.txt`
suffix keeps this supplemental input out of unrelated historical corpus sweeps.
This committed fixture supplements the reduced collection-read test in
`crates/vize_croquis/tests/fact_spec/main.rs`. It does not replace the fact-spec
corpus shard. That gate retains all 773 ant-design-vue/create-vue inputs and
uses the independently pinned layoutit-grid corpus as its positive unused
witness; the original and witness scopes are reported separately.
