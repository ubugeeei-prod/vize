## Summary

Two name-resolution cases differ from `@vue/compiler-vapor`:

1. **Self-reference by file name.** `<Tree>` inside `Tree.vue` compiles to `_resolveComponent("Tree")` without the `maybeSelfReference` flag. At runtime Vue warns `Failed to resolve component: Tree` and renders a literal `<tree>` element. The VDOM output of the same file is correct (`_resolveComponent("Tree", true)`).
2. **Directives declared in `<script setup>`.** `const vFocus = …` used as `v-focus` compiles to `_resolveDirective("focus")`, which only looks at globally/locally registered directives, so Vue warns `Failed to resolve directive: focus` and the directive never runs. The binding is even returned from setup (`__returned__ = { vFocus }`), it is just not used.

Same output from `@vizejs/native` `compileSfc(src, { vapor: true })`.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > Tree.vue <<'VUE'
<script setup vapor>
const { depth = 0 } = defineProps({ depth: Number });
const vMark = (el) => { el.dataset.mark = "yes"; };
</script>

<template>
  <div v-mark>{{ depth }}<Tree v-if="depth < 2" :depth="depth + 1" /></div>
</template>
VUE
npx vize@0.432.0 build --no-config -o out Tree.vue && grep -n -E 'resolveComponent\(|resolveDirective\(|withVaporDirectives' out/Tree.js
```

## Actual

```js
const _directive_mark = _resolveDirective("mark")
…
const _component_Tree = _resolveComponent("Tree")
```

Mounted with `createVaporApp(Tree)`: `<div>0<tree depth="1"></tree></div>` without `data-mark`, plus the two "Failed to resolve" warnings.

## Expected

`<div data-mark="yes">0<div data-mark="yes">1<div data-mark="yes">2</div></div></div>`. `@vue/compiler-sfc` (dev, non-inline) emits:

```js
const _component_Tree__self = _resolveComponent("Tree", true)
…
_withVaporDirectives(n4, [[_ctx.vMark]])
```
