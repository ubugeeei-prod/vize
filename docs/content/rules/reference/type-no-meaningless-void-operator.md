---
title: "type/no-meaningless-void-operator"
description: "Disallow void operators on expressions that are already void-like."
---

# `type/no-meaningless-void-operator`

Disallow void operators on expressions that are already void-like.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

logVoid() is already void, so the void operator does not change the result.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
void logVoid();
</script>
```

## Good

getNumber() returns a Promise, so void discards that thenable on purpose.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
void getNumber();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-meaningless-void-operator": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_meaningless_void_operator.rs#L5)
