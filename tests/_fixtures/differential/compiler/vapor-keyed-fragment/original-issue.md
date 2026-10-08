## Summary

`:key` on an element or component that is not in a `v-for` is the usual way to force a re-mount (reset local state, replay transitions). In Vapor output Vize drops it: no keyed fragment is emitted, the component is created once, and its local state survives a key change. `@vue/compiler-vapor` wraps the node in `createKeyedFragment(() => key, …)`.

Same output from `@vizejs/native` `compileSfc(src, { vapor: true })`.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup vapor>
import { ref } from "vue";
import Counter from "./Counter.vue";
const k = ref(1);
</script>

<template>
  <button @click="k++">reset</button>
  <Counter :key="k" />
</template>
VUE
cat > Counter.vue <<'VUE'
<script setup vapor>
import { ref } from "vue";
const n = ref(0);
</script>

<template>
  <i @click="n++">{{ n }}</i>
</template>
VUE
npx vize@0.432.0 build --no-config -o out App.vue && sed -n '/^function render/,/^}/p' out/App.js
```

## Actual

```js
const n1 = _createComponentWithFallback(_component_Counter, null, null, true)
```

Mounted: click `<i>` (shows `1`), then click "reset": `<i>` still shows `1`.

## Expected

After "reset", `<Counter>` is re-created and shows `0`, as with `@vue/compiler-sfc`, which emits (dev, non-inline):

```js
const n1 = _createKeyedFragment(() => (_ctx.k), () => {
  const n2 = _createComponent(_ctx.Counter)
  return n2
})
```
