### Area

Vite plugin

### Version

`main` at `14877bda8b96ae9b0e7e8e4bf587fda494a42b1a` (built from source); first seen on `@vizejs/vite-plugin@0.434.0`.

### Minimal reproduction

https://github.com/Hebilicious/vize-reproduction#2-svg-v-if-branches-and-dynamic-classstyle-are-broken

```vue
<script setup lang="ts" vapor>
import { computed } from "vue";

const props = defineProps<{ tier: number }>();
const color = computed(() => (props.tier === 1 ? "rgb(200, 0, 0)" : "rgb(0, 0, 200)"));
</script>

<template>
  <svg id="icon" class="icon" :class="`tier-${tier}`" :style="{ color }" viewBox="0 0 64 64" width="64" height="64">
    <g v-if="tier === 1" id="mark-1"><circle cx="32" cy="32" r="16" fill="currentColor" /></g>
    <g v-else id="mark-other"><rect x="16" y="16" width="32" height="32" fill="currentColor" /></g>
  </svg>
</template>
```

```sh
cd vize-reproduction/vize && pnpm install
pnpm exec vitest run --config vitest.browser.workaround.config.ts src/svg-icon.browser.test.ts
```

### Actual behavior

The circle never renders, and the root's `class` and `style` attributes read `icon,tier-1` and `[object Object]`:

```
AssertionError: expected 'http://www.w3.org/1999/xhtml' to be 'http://www.w3.org/2000/svg'
AssertionError: expected 'icon,tier-1' to be 'icon tier-1'
AssertionError: expected '[object Object]' not to contain '[object Object]'
```

Vize on `main`:

```js
const t0 = _template("<g id=\"mark-1\"><circle cx=\"32\" cy=\"32\" r=\"16\" fill=\"currentColor\"></circle></g>", true)
const t1 = _template("<g id=\"mark-other\"><rect x=\"16\" y=\"16\" width=\"32\" height=\"32\" fill=\"currentColor\"></rect></g>", true)
const t2 = _template("<svg id=\"icon\" viewBox=\"0 0 64 64\" width=\"64\" height=\"64\"></svg>", true, 1)
_setAttr(n0, "class", ["icon", `tier-${$props.tier}`])
_setAttr(n0, "style", { color: _ctx.color })
```

`template(html, root, ns)` parses inside an `<svg>` wrapper only when `ns === 1`. The root `<svg>` template passes it, but the `v-if` branch templates don't, so each `<g>` is created as an HTML element. Dynamic `class`/`style` on SVG elements compile to `setAttr`, which writes the raw array or object.

### Expected behavior

As with `@vue/compiler-vapor`:

```js
const t0 = _template("<g id=mark-1><circle cx=32 cy=32 r=16 fill=currentColor>", 2, 1);
const t1 = _template("<g id=mark-other><rect x=16 y=16 width=32 height=32 fill=currentColor>", 2, 1);
const t2 = _template('<svg id=icon viewBox="0 0 64 64" width=64 height=64>', 1, 1);
_setClass(n5, ["icon", `tier-${$props.tier}`], true);
_setStyle(n5, { color: _ctx.color });
```

The same fix was merged in my fork as [Hebilicious/vize#4](https://github.com/Hebilicious/vize/pull/4). Draft upstream PR with the fix: #8619.

### Environment

- OS: Linux (Arch)
- Architecture: x86_64
- Node.js: 24.14.0
- Package manager: pnpm 12.1.0
- Rust: 1.99.0 (to build `main`)
- Framework: vue / @vue/runtime-vapor 3.6.0-rc.10, vite 8.3.2, vitest 4.1.11, @vitest/browser-playwright 4.1.11

### Before submitting

- [x] I checked existing fix reports and pull requests.
- [x] This is not a private security report.
