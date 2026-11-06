---
title: "script/no-with-defaults"
description: "Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)."
---

# `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`withDefaults` wraps the typed props declaration solely to supply `count` and `name` defaults, instead of the Vue 3.5+ destructuring-default style preferred here.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## Good

The destructuring pattern puts `count = 0` and `name = "Ada"` beside their bindings and removes the `withDefaults` wrapper.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-with-defaults": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41)
