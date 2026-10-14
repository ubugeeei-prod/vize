# `whitespace: "vue2-preserve"` for Vue 2.7 parity

Issue: [#8259](https://github.com/ubugeeei-prod/vize/issues/8259).

Vue 2 / vue-loader 15 / Nuxt 2 apps are compiled with Vue 2.7
`whitespace: "preserve"`, which differs from Vue 3 `preserve` around
interpolations: Vue 2 parses a run of text and interpolations as one text node,
so whitespace next to an interpolation, and at the start or end of an element,
is kept as written. Neither existing mode reproduced it (`"vue2-line-breaks"`
only keeps a line break after text).

Decision: add an explicit opt-in mode, `"vue2-preserve"`, accepted everywhere
`compiler.whitespace` is (vize.config, Vite and Nuxt options, CLI, native and
wasm bindings). It reports the `Preserve` strategy to later stages, so DOM, SSR
and Vapor keep their preserve handling (legacy lanes), and only the parser's
post-pass differs: a text/interpolation run that holds any interpolation or
non-whitespace text is kept verbatim; a whitespace-only run is dropped at the
edges of a child list and condensed to one space between elements; `<pre>` and
`<textarea>` keep their content. The mode adds its own bit to the native batch
and CLI build cache keys.

Vue's `nodeTransforms` hook was not added: a typed whitespace mode is the
smaller, testable contract, and the reference outputs come from
`@vue/compiler-sfc@2.7.16` (fixture `whitespace-vue2-preserve-8259`).
