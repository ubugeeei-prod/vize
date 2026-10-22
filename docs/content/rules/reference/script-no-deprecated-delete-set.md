---
title: "script/no-deprecated-delete-set"
---

# `script/no-deprecated-delete-set`

Disallow Vue 2 $set and $delete, which were removed in Vue 3

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
        "script/no-deprecated-delete-set": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`created()` calls the removed `this.$set` helper to add `ready`.

```vue
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## Good

The same field is assigned directly with `this.item.ready = true`.

```vue
<script lang="ts">
export default {
  created() {
    this.item.ready = true
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L41) · [All rules](../all.md)
