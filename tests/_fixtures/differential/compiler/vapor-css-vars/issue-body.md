## Summary

For a Vapor SFC (`<script setup vapor>` or `--vapor`) with `v-bind()` in `<style>`, Vize imports and calls the VDOM helper `useCssVars`. In a Vapor component it finds no VDOM instance: Vue warns `useCssVars is called without current active component instance.` and none of the `--<hash>-*` custom properties are set, so every `v-bind()` value in the stylesheet resolves to nothing. `@vue/compiler-sfc` uses `useVaporCssVars` for Vapor components.

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
const color = ref("red");
</script>

<template>
  <p class="x" @click="color = 'blue'">text</p>
</template>

<style scoped>
.x { color: v-bind(color); }
</style>
VUE
npx vize@0.432.0 build --no-config -f json -o out App.vue && node -e 'const j=require("./out/App.json"); console.log(j.code.split("\n").filter(l=>/CssVars/.test(l)).join("\n")); console.log(j.css)'
```

## Actual

```js
import { useCssVars as _useCssVars, unref as _unref } from 'vue'
…
_useCssVars(_ctx => ({
  "<hash>-color": (color.value)
}))
```

Mounted with `createVaporApp`: `<p class="x" data-v-…>` has no `style`, and the warning above is logged.

## Expected

`<p class="x" style="--<hash>-color: red;">`, updated to `blue` on click. `@vue/compiler-sfc` emits:

```js
import { useVaporCssVars as _useVaporCssVars, defineVaporComponent as _defineVaporComponent } from "vue";
    _useVaporCssVars((_ctx) => ({ "<hash>-color": color.value }));
```

