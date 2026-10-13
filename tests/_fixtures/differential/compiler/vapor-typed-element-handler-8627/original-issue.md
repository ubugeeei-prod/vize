### Area

Vite plugin

### Version

`main` at `14877bda8b96ae9b0e7e8e4bf587fda494a42b1a` (built from source); first seen on `@vizejs/vite-plugin@0.434.0`.

### Minimal reproduction

```vue
<script setup lang="ts" vapor>
import { shallowRef } from "vue";

const clicked = shallowRef("none");
</script>

<template>
  <div>
    <button id="typed-click" type="button" @click="(event: MouseEvent) => (clicked = event.type)">Click</button>
    <span id="typed-clicked">{{ clicked }}</span>
  </div>
</template>
```

Mounted with `createVaporApp` in Vitest Browser Mode with `@vizejs/vite-plugin` (`vapor: true`) built from `main`: after a click, `#typed-clicked` stays `none` (`AssertionError: expected 'none' to be 'click'`). The same test passes with `@vitejs/plugin-vue`.

The component-listener half of this was reported in #7885 and fixed by #8133 ([reproduction 9](https://github.com/Hebilicious/vize-reproduction#9-an-inline-handler-with-a-typed-parameter-never-runs) now passes on `main`). Native element listeners still use the JavaScript-only classifier.

### Actual behavior

```js
n0.$evtclick = _createInvoker(($event: any) => ((event: _ctx.MouseEvent) => (_ctx.clicked = event.type)
))
```

The annotated arrow fails the JavaScript parse that decides whether the handler is a function, so it is wrapped as a statement that returns the handler instead of calling it.

### Expected behavior

The arrow is the handler, as with `@vue/compiler-vapor`:

```js
_on(n0, "click", (event: MouseEvent) => (_ctx.clicked = event.type))
```

The same fix was merged in my fork as [Hebilicious/vize#18](https://github.com/Hebilicious/vize/pull/18). Draft upstream PR with the fix: #8622.

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
