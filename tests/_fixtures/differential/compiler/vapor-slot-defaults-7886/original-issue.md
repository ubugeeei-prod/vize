## Summary

In Vapor output, a default value in a destructured `v-slot` / `#name` scope is ignored: `{{ label }}` reads `_slotProps0.label` directly and renders empty when the slot does not pass `label`. VDOM output keeps the destructuring (`({ label = 'x' }) => …`), and `@vue/compiler-vapor` emits `_getDefaultValue(_slotProps0.label, () => ('x'))`.

Same output from `@vizejs/native` `compileSfc(src, { vapor: true })`.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup vapor>
import Child from "./Child.vue";
</script>

<template>
  <Child v-slot="{ label = 'fallback', count }">{{ label }}:{{ count }}</Child>
</template>
VUE
cat > Child.vue <<'VUE'
<script setup vapor>
const n = 1;
</script>

<template>
  <slot :count="n" />
</template>
VUE
npx vize@0.432.0 build --no-config -o out App.vue && grep -n '_slotProps0' out/App.js
```

## Actual

```js
"default": (_slotProps0) => {
  …
  _renderEffect(() => _setText(n0, _toDisplayString(_slotProps0.label) + ":" + _toDisplayString(_slotProps0.count)))
```

Mounted: `:1`.

## Expected

`fallback:1`, with the default applied as in `@vue/compiler-sfc`:

```js
_setText(n0, _toDisplayString(_getDefaultValue(_slotProps0.label, () => ('fallback'))) + ":" + _toDisplayString(_slotProps0.count))
```
