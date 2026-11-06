---
title: "script/no-multiple-slot-args"
description: "Disallow passing more than one argument to a scoped-slot function call."
---

# `script/no-multiple-slot-args`

Disallow passing more than one argument to a scoped-slot function call.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The slot calls pass multiple positional arguments or spread an unknown argument list. Vue slots receive one props object, not a positional parameter list.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

## Good

`{ foo, bar }` combines the data into one argument; `slotProps` and the argument-free call also stay within the supported slot-call shape.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-multiple-slot-args": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61)
