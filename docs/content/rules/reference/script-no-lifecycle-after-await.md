---
title: "script/no-lifecycle-after-await"
description: "Disallow lifecycle hooks registered after an await in setup()."
---

# `script/no-lifecycle-after-await`

Disallow lifecycle hooks registered after an await in setup().

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`setup()` calls `onMounted()` after `await loadData()`; with no current component instance at that point, the hook is never registered.

```vue annotate="remove:5"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    await loadData()
    onMounted(() => {})
  },
}
</script>
```

## Good

`onMounted()` is registered before the first `await`, while `setup()` still runs synchronously for the component.

```vue annotate="add:6"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    onMounted(() => {})
    await loadData()
  },
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-lifecycle-after-await": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45)
