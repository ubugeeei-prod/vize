---
title: "script/one-component-per-file"
description: "Disallow more than one component definition in a file."
---

# `script/one-component-per-file`

Disallow more than one component definition in a file.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The file exports `Parent` and also defines an inline `Child` component in `components`.

```vue annotate="remove:1,2,3"
<script setup lang="ts">
const First = defineComponent({ name: 'First' })
const Second = defineComponent({ name: 'Second' })
</script>
```

## Good

The file defines only `Parent`.

```vue annotate="add:1,2,3,4"
<script lang="ts">
export default {
  name: 'Parent'
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
      "script/one-component-per-file": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L26)
