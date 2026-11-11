---
title: "script/require-explicit-slots"
description: "Require slots consumed via useSlots() to be explicitly typed with defineSlots<...>()."
---

# `script/require-explicit-slots`

Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;().

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The typed `defineProps<{ id: number }>()` establishes TypeScript syntax, but setup uses `useSlots()` without a `defineSlots` declaration. The rule therefore finds consumed slots without an explicit slot contract.

```vue annotate="remove:2,4,5,6"
<script setup lang="ts">
defineSlots<{ default(): unknown }>()
</script>
<template>
  <div><slot /><slot name="footer" /></div>
</template>
```

## Good

`defineSlots` declares a `default` slot whose props include `msg: string`; `useSlots()` now appears alongside an explicit typed slot contract.

```vue annotate="add:2,3"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-slots": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vue

`defineSlots()` with no type argument declares no slots. `Readonly`, `Partial`, and `Required` keep the inner names, and `Record` with a union of string-literal keys uses those keys. An imported type is enumerated when the cross-file type world can resolve it. A type that still cannot be enumerated stays unchecked, so an unresolved import is not treated as an empty declaration.

A repeated name inside one type literal, interface, or alias is reported. The same name on both sides of an intersection is not. An index signature does not name a slot, so a rendered outlet next to only an index signature is reported.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L58)
