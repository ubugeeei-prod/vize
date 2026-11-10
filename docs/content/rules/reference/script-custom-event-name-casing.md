---
title: "script/custom-event-name-casing"
description: "Enforce configured casing for emitted custom event names."
---

# `script/custom-event-name-casing`

Enforce configured casing for emitted custom event names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | No | [Rule options](../options.md) |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The emitted string `my-event` contains a hyphen and violates the default camelCase event naming policy.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

## Good

Both the declaration and call use `myEvent`, preserving agreement while satisfying the global camelCase default. The `opinionated` preset selects kebab-case instead, and explicit rule options can also select lowercase names such as `transitionstart`.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/custom-event-name-casing": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L74)
