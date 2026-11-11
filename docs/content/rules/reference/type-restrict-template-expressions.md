---
title: "type/restrict-template-expressions"
description: "Require template literal expressions to be string-compatible."
---

# `type/restrict-template-expressions`

Require template literal expressions to be string-compatible.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The template interpolates an array, which is not a string-compatible expression.

```vue annotate="remove:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const listText = `${numbers}`;
</script>
```

## Good

The template interpolates label, which is a string.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
const listText = `${label}`;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-template-expressions": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_template_expressions.rs#L5)
