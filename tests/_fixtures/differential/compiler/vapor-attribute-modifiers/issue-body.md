## Summary

Two attribute-binding differences from `@vue/compiler-vapor`:

1. **`.attr` is ignored for names that are also DOM properties.** `:value.attr` on `<input>` and `:title.attr` are emitted as `setProp(…)` (the DOM property), so `<input>` gets no `value` attribute. `.attr` on other names (`:foo-bar.attr`) correctly uses `setAttr`.
2. **Merge order with `v-bind="obj"`.** When the element also has a `.attr` / `.prop` binding, a static attribute written *after* `v-bind="obj"` is put in the template and then overwritten by the object (`setDynamicProps(n, [obj])`), so the object wins. In Vue the later attribute wins (https://v3-migration.vuejs.org/breaking-changes/v-bind.html). Without a `.attr` / `.prop` binding on the element, the order is correct.

Same output from `@vizejs/native` `compileSfc(src, { vapor: true })`.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup vapor>
const v = "x";
const attrs = { id: "from-object" };
</script>

<template>
  <input :value.attr="v" />
  <div :title.attr="v" v-bind="attrs" id="static"></div>
</template>
VUE
npx vize@0.432.0 build --no-config -o out App.vue && grep -n -E '_template|_set' out/App.js
```

## Actual

```js
const t1 = _template("<div id=\"static\"></div>")
…
_setProp(n0, "value", _ctx.v)
_setProp(n1, "title", _ctx.v)
_setDynamicProps(n1, [_ctx.attrs])
```

Mounted: `<input><div id="from-object" title="x"></div>`.

## Expected

`<input value="x"><div title="x" id="static"></div>`. `@vue/compiler-sfc` emits (dev, non-inline):

```js
_setAttr(n0, "value", _ctx.v)
_setDynamicProps(n1, [{ "^title": _ctx.v }, _ctx.attrs, { id: "static" }])
```
