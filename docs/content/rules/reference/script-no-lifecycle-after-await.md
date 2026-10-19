---
title: "script/no-lifecycle-after-await"
---

# `script/no-lifecycle-after-await`

Disallow lifecycle hooks registered after an await in setup()

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-lifecycle-after-await": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`setup()` calls `onMounted()` after `await loadData()`; with no current component instance at that point, the hook is never registered.

```vue
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

```vue
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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45) · [All rules](../all.md)
