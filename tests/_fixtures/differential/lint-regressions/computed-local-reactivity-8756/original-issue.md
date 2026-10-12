### Summary

Inside a `computed()` getter, reading `.value` into a local and spreading a field of it are reported as reactivity loss. The getter re-runs whenever the ref changes, so the local is always current. This looks like the case #7244 fixed for a direct spread of `.value`, with the value read into a local first.

### Version

- `vize` 0.443.0 (npm, bundled `@typescript/typescript-darwin-arm64` 7.0.2)
- Node v26.10.0, macOS arm64

`vize check` reports no type errors for the same file.

### Reproduction

`vize.config.json`

```json
{
  "linter": {
    "preset": "opinionated",
    "typeAware": true
  }
}
```

`tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "lib": ["ESNext", "DOM"]
  },
  "include": ["*.vue", "*.ts"]
}
```

`ComputedLocal.vue`

```vue
<script setup lang="ts">
import { computed, ref } from "vue";

const held = ref<{ id: number; shape: { rot: number } } | null>(null);

// The getter re-runs whenever `held` changes, so the local is always current.
const shown = computed(() =>
  [1, 2].map((id) => {
    const h = held.value;
    return h?.id === id ? { id, ...h.shape } : { id, rot: 0 };
  }),
);
</script>

<template>
  <p v-for="s in shown" :key="s.id">{{ s.rot }}</p>
</template>
```

`vize lint --format plain ComputedLocal.vue`

```
  ComputedLocal.vue:9:15 warning type/no-reactivity-loss Assigning 'held.value' to 'h' stores a plain snapshot
  ComputedLocal.vue:10:33 warning type/no-reactivity-loss Spreading 'h.shape' creates a non-reactive copy
```