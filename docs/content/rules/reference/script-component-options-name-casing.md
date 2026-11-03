---
title: "script/component-options-name-casing"
description: "Enforce PascalCase for the component `name` option."
---

# `script/component-options-name-casing`

Enforce PascalCase for the component `name` option.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The component option `name: 'my-component'` is kebab-case, whereas this rule requires a literal component name in PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## Good

`MyComponent` begins with an uppercase letter and contains only alphanumeric characters, satisfying the name check.

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44)
