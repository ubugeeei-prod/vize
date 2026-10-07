## Environment

- vize: 0.163.0
- Vue: 3.5.x

## Minimal repro

```vue
<!-- src/Provider.vue -->
<script setup lang="ts">
import { provide, ref } from 'vue'

const count = ref(0)
provide('count', count)
</script>

<template>
  <slot />
</template>
```

```vue
<!-- src/Consumer.vue -->
<script setup lang="ts">
import { inject } from 'vue'

const count = inject('count')
</script>

<template>
  <div>{{ count }}</div>
</template>
```

```vue
<!-- src/App.vue -->
<script setup lang="ts">
import Consumer from './Consumer.vue'
import Provider from './Provider.vue'
</script>

<template>
  <Provider>
    <Consumer />
  </Provider>
</template>
```

Run:

```bash
pnpm add -D vize@0.163.0 vue@3.5.17 typescript
pnpm exec vize lint --cross-file --cross-file-tree --format plain src
```

## Actual

Vize reports both sides as disconnected:

```text
src/Consumer.vue:4:15 error cross-file vize:croquis/cf/unmatched-inject: inject('count') has no matching provide() in any ancestor component
src/Provider.vue:5:1 warning cross-file vize:croquis/cf/unused-provide: provide('count') is not used by any descendant component
```

It also does not print a useful provide/inject tree despite `--cross-file-tree` being passed.

## Expected

`Consumer` should be treated as a slot descendant of `Provider` for provide/inject ancestry. The `inject('count')` should match the `provide('count', count)` in `Provider.vue`, and `provide('count')` should not be reported as unused.

## Proposed direction

The cross-file component graph should model slot-projected children as runtime descendants of the receiving component, not only lexical children inside the provider component's own SFC. For provide/inject analysis, build ancestry edges from parent component usage sites in templates, then route default and named slot children through the component instance that renders `<slot>`. The tree printer should use the same graph so `--cross-file-tree` exposes the matched provider/consumer path and makes graph gaps visible.
