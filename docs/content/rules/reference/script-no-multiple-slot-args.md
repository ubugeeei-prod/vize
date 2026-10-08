---
title: "script/no-multiple-slot-args"
---

# `script/no-multiple-slot-args`

Disallow passing more than one argument to a scoped-slot function call

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The slot calls pass multiple positional arguments or spread an unknown argument list. Vue slots receive one props object, not a positional parameter list.

```vue
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

```vue
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [All rules](../all.md)
