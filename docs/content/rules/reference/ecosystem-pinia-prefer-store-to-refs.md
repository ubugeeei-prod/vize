---
title: "ecosystem/pinia-prefer-store-to-refs"
description: "Prefer storeToRefs() when destructuring Pinia stores."
---

# `ecosystem/pinia-prefer-store-to-refs`

Prefer storeToRefs() when destructuring Pinia stores.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

Destructuring `name` directly from the store separates the value from its reactive store access.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

## Good

The store remains intact and storeToRefs creates a reactive reference for name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20)
