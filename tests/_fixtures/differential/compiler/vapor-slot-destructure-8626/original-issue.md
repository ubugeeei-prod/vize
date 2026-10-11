### Area

Vite plugin

### Version

`main` at `14877bda8b96ae9b0e7e8e4bf587fda494a42b1a` (built from source); first seen on `@vizejs/vite-plugin@0.434.0`.

### Minimal reproduction

https://github.com/Hebilicious/vize-reproduction#7-a-renamed-or-nested-slot-prop-destructure-reads-the-wrong-key

```vue
<!-- SlotOwner.vue -->
<script setup lang="ts" vapor>
const trigger = { id: "slot-trigger", label: "Open" };
</script>

<template>
  <slot :props="trigger" />
</template>

<!-- SlotConsumer.vue -->
<script setup lang="ts" vapor>
import SlotOwner from "./SlotOwner.vue";
</script>

<template>
  <div>
    <SlotOwner v-slot="{ props: trigger }">
      <button :id="trigger.id" type="button">{{ trigger.label }}</button>
    </SlotOwner>
    <SlotOwner v-slot="{ props: { label } }">
      <span id="nested-label">{{ label }}</span>
    </SlotOwner>
  </div>
</template>
```

```sh
cd vize-reproduction/vize && pnpm install
pnpm exec vitest run --config vitest.browser.workaround.config.ts src/components.browser.test.ts -t "7."
```

### Actual behavior

Mounting throws:

```
TypeError: Cannot read properties of undefined (reading 'id')
```

The local name is read as if it were the slot prop's own key:

```js
_setProp(n3, "id", _slotProps0.trigger.id)
_setText(x3, _toDisplayString(_slotProps0.trigger.label))
_renderEffect(() => _setText(x4, _toDisplayString(_slotProps1.label)))
```

A plain `v-slot="{ props }"` works, and defaults (`{ label = 'x' }`, #7886) work. Only a rename or a nested pattern breaks.

### Expected behavior

Each name is read through the key it was destructured from, as with `@vue/compiler-vapor`:

```js
const _trigger = _slotProps0.props;
_setProp(n0, "id", _trigger.id);
_renderEffect(() => _setText(x2, _toDisplayString(_slotProps0.props.label)));
```

The same fix was merged in my fork as [Hebilicious/vize#14](https://github.com/Hebilicious/vize/pull/14). Draft upstream PR with the fix: #8621.

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
