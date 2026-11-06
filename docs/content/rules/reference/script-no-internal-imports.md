---
title: "script/no-internal-imports"
description: "Disallow importing from Vue internal modules."
---

# `script/no-internal-imports`

Disallow importing from Vue internal modules.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

Both imports address internal `dist` files rather than Vue’s public package entry point, coupling the component to build-file paths.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

## Good

Importing the required helpers from `vue` removes the dependency on internal distribution file locations.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-internal-imports": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28)
