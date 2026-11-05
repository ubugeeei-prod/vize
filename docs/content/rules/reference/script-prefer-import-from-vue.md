---
title: "script/prefer-import-from-vue"
description: "Prefer importing from 'vue' instead of internal packages."
---

# `script/prefer-import-from-vue`

Prefer importing from 'vue' instead of internal packages.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Yes | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`ref` and `h` are imported from the internal `@vue/runtime-core` and `@vue/runtime-dom` packages rather than the public `vue` package.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## Good

The public helpers are imported from `vue`. Private `isString` retains `@vue/shared`, because `vue` does not export it. Namespace, default and mixed public/private imports keep a warning without an automatic fix.

```vue annotate="add:2,3"
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
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
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36)
