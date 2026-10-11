### Area

Vite plugin

### Version

`main` at `14877bda8b96ae9b0e7e8e4bf587fda494a42b1a` (built from source); first seen on `@vizejs/vite-plugin@0.434.0`.

### Minimal reproduction

https://github.com/Hebilicious/vize-reproduction#5-a-nested-component-receives-its-owners-fallthrough-attributes

```vue
<!-- FallthroughOuter.vue, mounted with { "data-owner": "outer" } -->
<script setup lang="ts" vapor>
import FallthroughInner from "./FallthroughInner.vue";
</script>

<template>
  <section id="outer">
    <FallthroughInner />
  </section>
</template>

<!-- FallthroughInner.vue -->
<script setup lang="ts" vapor></script>

<template>
  <span id="inner">inner</span>
</template>
```

```sh
cd vize-reproduction/vize && pnpm install
pnpm exec vitest run --config vitest.browser.workaround.config.ts src/components.browser.test.ts -t "5."
```

### Actual behavior

`data-owner` lands on `#outer` and also on `#inner`:

```
AssertionError: expected true to be false // Object.is equality
```

Every created component passes `isSingleRoot = true`, including one nested inside the root element:

```js
const n0 = _createComponentWithFallback(_component_FallthroughInner, null, null, true)
```

In a real app, a tab panel's `id` and `role` landed on a dialog rendered inside it.

### Expected behavior

Only the render's single root component (directly, through a root `v-if` branch, or through a root keyed fragment) receives fallthrough attrs, as with `@vue/compiler-vapor`:

```js
const n0 = _createComponent(FallthroughInner)
```

The same fix was merged in my fork as [Hebilicious/vize#10](https://github.com/Hebilicious/vize/pull/10). Draft upstream PR with the fix: #8620.

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
