---
title: "script/no-expose-after-await"
description: "Disallow expose() and defineExpose() after an await."
---

# `script/no-expose-after-await`

Disallow expose() and defineExpose() after an await.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`defineExpose()` runs after a top-level `await`, when there is no current component instance, so nothing is exposed.

```vue annotate="remove:2"
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## Good

`defineExpose()` runs before the first `await`, while the component instance is still current.

```vue annotate="add:3"
<script setup lang="ts">
defineExpose({ reload })
await loadData()
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
        "script/no-expose-after-await": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34)
