---
title: "script/no-deprecated-delete-set"
description: "Disallow Vue 2 $set and $delete, which were removed in Vue 3."
---

# `script/no-deprecated-delete-set`

Disallow Vue 2 $set and $delete, which were removed in Vue 3.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`created()` calls the removed `this.$set` helper to add `ready`.

```vue annotate="remove:4"
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

```vue annotate="add:4"
<script lang="ts">
export default {
  created() {
    this.item.ready = true
  }
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
      "script/no-deprecated-delete-set": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L38)
