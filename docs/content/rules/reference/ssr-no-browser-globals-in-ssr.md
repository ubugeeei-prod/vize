---
title: "ssr/no-browser-globals-in-ssr"
description: "Disallow browser-only globals in SSR context."
---

# `ssr/no-browser-globals-in-ssr`

Disallow browser-only globals in SSR context.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Setup reads `window.innerWidth` immediately, although `window` does not exist when the component runs on the server.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

## Good

The initial width is a server-safe ref value, and the browser access moves into `onMounted`, which runs on the client rather than during SSR setup.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ssr/no-browser-globals-in-ssr": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [SSR rules](../ssr.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157)
