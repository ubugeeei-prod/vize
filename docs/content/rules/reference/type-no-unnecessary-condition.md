---
title: "type/no-unnecessary-condition"
description: "Disallow conditions that are always truthy, always falsy, never, or always nullish."
---

# `type/no-unnecessary-condition`

Disallow conditions that are always truthy, always falsy, never, or always nullish.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

value is a string, so it is never nullish and the ?? fallback never runs.

```vue annotate="remove:2"
<script setup lang="ts">
function orDefault(value: string): string {
  return value ?? "x";
}
</script>
```

## Good

value may be undefined, so the ?? fallback supplies a string when it is missing.

```vue annotate="add:2"
<script setup lang="ts">
function orDefault(value: string | undefined): string {
  return value ?? "x";
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-condition": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_condition.rs#L5)
