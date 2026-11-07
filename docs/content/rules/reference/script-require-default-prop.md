---
title: "script/require-default-prop"
description: "Require a default value for every optional, non-Boolean prop."
---

# `script/require-default-prop`

Require a default value for every optional, non-Boolean prop.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`name` and `age` are optional non-Boolean runtime props without defaults, leaving their omitted-input values unspecified.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

## Good

`name` receives `default: ''`. `enabled` uses Boolean’s implicit false default, and required `id` needs no fallback, illustrating both exemptions.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
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
      "script/require-default-prop": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L59)
