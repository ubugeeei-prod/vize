---
title: "vue/require-component-registration"
description: "Require explicit import or registration for components."
---

# `vue/require-component-registration`

Require explicit import or registration for components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.

## Bad

`MissingWidget` is neither registered nor included in the configured global-component allowlist.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

## Good

`MyButton` is listed in the example's `globals` option. That option exempts a known global component; it does not register or import it.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-registration": "warn"
    },
    "ruleOptions": {
      "vue/require-component-registration": {
        "globals": [
          "MyButton",
          "MyIcon"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L59)
