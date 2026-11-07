---
title: "script/no-unused-emit-declarations"
description: "Flag declared events that are never emitted."
---

# `script/no-unused-emit-declarations`

Flag declared events that are never emitted.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`defineEmits` declares both `change` and `unused`, but the captured `emit` function only emits the literal event `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## Good

Removing `unused` makes the declared event list match the observed emission. The example uses a captured, unescaped emit binding so this local usage conclusion is available.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-unused-emit-declarations": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L82)
