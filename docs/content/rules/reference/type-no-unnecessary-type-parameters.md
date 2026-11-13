---
title: "type/no-unnecessary-type-parameters"
description: "Disallow type parameters that are used only once in their signature."
---

# `type/no-unnecessary-type-parameters`

Disallow type parameters that are used only once in their signature.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

T appears only once in the signature of logValue, so it relates nothing to anything.

```vue annotate="remove:2"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
logValue(1);
</script>
```

## Good

Typing value as unknown accepts the same arguments without a type parameter.

```vue annotate="add:2"
<script setup lang="ts">
function logValue(value: unknown): void {
  console.info(value);
}
logValue(1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-parameters": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_parameters.rs#L5)
