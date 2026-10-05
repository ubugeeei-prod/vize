## Summary

The common slot-forwarding pattern

```vue
<Inner>
  <template v-for="(_, name) in $slots" #[name]="scope">
    <slot :name="name" v-bind="scope" />
  </template>
</Inner>
```

compiles in SSR mode to a `_createSlots(…, [_renderList(_ctx.$slots, …)])` call, but the file imports `ssrRenderList` from `@vue/server-renderer` and never imports `renderList` from `vue`. Server rendering throws `ReferenceError: _renderList is not defined`. The client (VDOM and Vapor) output of the same file works.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/server-renderer` 3.6.0-rc.10
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > Forward.vue <<'VUE'
<script setup>
import Inner from "./Inner.vue";
</script>

<template>
  <Inner>
    <template v-for="(_, name) in $slots" #[name]="scope">
      <slot :name="name" v-bind="scope" />
    </template>
  </Inner>
</template>
VUE
npx vize@0.432.0 build --no-config --ssr -o out Forward.vue && grep -n -E '^import|renderList' out/Forward.js
```

## Actual

```js
1:import { ssrRenderComponent as _ssrRenderComponent, ssrRenderSlot as _ssrRenderSlot, ssrRenderList as _ssrRenderList } from "@vue/server-renderer"
2:import { renderSlot as _renderSlot, createSlots as _createSlots, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"
9:    _renderList(_ctx.$slots, (_, name) => {
```

`renderToString(createSSRApp(App))` with `App` rendering `<Forward>` (and `Inner.vue` rendering `<div><slot /></div>`) fails with `ReferenceError: _renderList is not defined` in `ssrRender`.

## Expected

`renderList` is imported from `vue` (as `@vue/compiler-sfc` does for this template with `ssr: true`), and the forwarded slots render, e.g. `<div><!--[--><!--[-->…<!--]--><!--]--></div>`.
