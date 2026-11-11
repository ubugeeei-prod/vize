---
title: "type/no-unnecessary-template-expression"
description: "Disallow template literals that wrap a single expression unnecessarily."
---

# `type/no-unnecessary-template-expression`

Disallow template literals that wrap a single expression unnecessarily.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The template contains only label, so it does not build a new string.

```vue annotate="remove:3"
<script setup lang="ts">
const label = "a";
const wrapped = `${label}`;
</script>
```

## Good

Assigning label uses the string without a template wrapper.

```vue annotate="add:3"
<script setup lang="ts">
const label = "a";
const wrapped = label;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-template-expression": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_template_expression.rs#L5)
