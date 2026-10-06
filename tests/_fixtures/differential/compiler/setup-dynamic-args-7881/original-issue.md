## Summary

In VDOM output for `<script setup>`, a dynamic argument of `v-on` or `v-bind` that names a setup binding is emitted as `_ctx.<name>`. With the inline render function, setup bindings are not on `_ctx`, so the argument evaluates to `undefined`: both the listener key and the attribute name become `""`, and Vue warns `Property "evt" was accessed during render but is not defined on instance` on every render. The listener never fires.

The value of the same directive is resolved correctly (`n.value++`), and the dynamic argument of a custom directive is too (`dir.value`); only the key expression of `v-on` / `v-bind` is wrong. Vapor output is not affected.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup>
import { ref } from "vue";
const evt = ref("click");
const attr = ref("title");
const n = ref(0);
</script>

<template>
  <button @[evt]="n++" :[attr]="'tip'">{{ n }}</button>
</template>
VUE
npx vize@0.432.0 build --no-config -o out App.vue && cat out/App.js
```

## Actual

```js
return (_openBlock(), _createElementBlock("button", {
  [_toHandlerKey(_ctx.evt)]: _cache[0] || (_cache[0] = $event => (n.value++)),
  [_ctx.attr || ""]: 'tip'
}, _toDisplayString(n.value), 17 /* TEXT, FULL_PROPS */))
```

Mounted, clicking the button does nothing and the `Property "evt" was accessed during render but is not defined on instance` warning is logged.

## Expected

The setup bindings are used, as `@vue/compiler-sfc` (`compileScript(…, { inlineTemplate: true })`) emits:

```js
[_toHandlerKey(evt.value)]: $event => (n.value++)
…
{ [attr.value || ""]: 'tip' }
```
