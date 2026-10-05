## Summary

Two built-in components are mis-compiled in SSR mode:

1. **`<Suspense>`**: the `#fallback` slot content is pushed inside the `default` branch of `ssrRenderSuspense`, so the server HTML contains the resolved content **followed by the fallback** (`<p>done</p>loading`). The fallback should never be server-rendered.
2. **`<TransitionGroup tag="ul">`**: emitted as `_ssrRenderComponent(_resolveComponent("TransitionGroup"), …)`. The runtime warns `Failed to resolve component: TransitionGroup` and the HTML contains a literal `<TransitionGroup tag="ul">` element instead of `<ul>`.

Client output (VDOM and Vapor) handles both correctly.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/server-renderer` / `@vue/compiler-sfc` 3.6.0-rc.10
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup>
import { Suspense } from "vue";
import Async from "./Async.vue";
</script>

<template>
  <Suspense>
    <Async />
    <template #fallback>loading</template>
  </Suspense>
  <TransitionGroup tag="ul"><li key="1">1</li></TransitionGroup>
</template>
VUE
cat > Async.vue <<'VUE'
<script setup>
const v = await Promise.resolve("done");
</script>

<template>
  <p>{{ v }}</p>
</template>
VUE
npx vize@0.432.0 build --no-config --ssr -o out App.vue && sed -n '/function ssrRender/,/^}/p' out/App.js
```

## Actual

```js
_ssrRenderSuspense(_push, {
  default: () => {
    _push(_ssrRenderComponent($setup.Async, null, null, _parent))
    _push(`loading`)
  },
  _: 1
})
_push(_ssrRenderComponent(_resolveComponent("TransitionGroup"), { tag: "ul" }, { … }, _parent))
```

`renderToString`: `<!--[--><p>done</p>loading<TransitionGroup tag="ul"><li>1</li></TransitionGroup><!--]-->` and `[Vue warn]: Failed to resolve component: TransitionGroup`.

## Expected

`<!--[--><p>done</p><ul><li>1</li></ul><!--]-->`, as with `@vue/compiler-sfc` (`ssr: true`): the fallback is not part of the SSR `default` branch, and `<TransitionGroup>` is compiled by the SSR transform for the built-in (rendering its `tag` element and children).
