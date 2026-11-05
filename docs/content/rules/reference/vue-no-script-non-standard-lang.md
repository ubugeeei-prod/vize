---
title: "vue/no-script-non-standard-lang"
description: "Discourage non-standard script lang values."
---

# `vue/no-script-non-standard-lang`

Discourage non-standard script lang values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Not implemented for SFC lint | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

> [!WARNING]
> Current support: `no-sfc-finding`. Vize accepts this rule ID in configuration, but the SFC linter does not report it yet. Good shows the intended convention.

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

## Bad

The script uses CoffeeScript syntax under lang=coffee. The current SFC path does not emit this catalog rule for that language.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## Good

The script uses an ordinary TypeScript declaration with lang=ts, illustrating the intended language convention.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

## Configured ID (currently no SFC finding)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44)
