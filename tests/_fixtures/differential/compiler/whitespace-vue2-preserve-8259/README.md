# `whitespace: "vue2-preserve"` (#8259)

`App.vue.txt` is the reproduction from issue #8259. `vue27-preserve-render.txt`
is the render body that `@vue/compiler-sfc@2.7.16` produces for its template
with `compileTemplate({ compilerOptions: { whitespace: "preserve" } })`.

Vue 2 merges each run of text and interpolations into one `_v(...)` text node,
so the whitespace next to `{{ n }}` and `{{ b }}` is kept as written, while
whitespace-only runs between elements become one space and are dropped at the
edges of a child list. `crates/vize/tests/build_vue2_preserve_cli.rs` checks
that `vize build` with `vize.config.json.txt` emits the same text runs.
