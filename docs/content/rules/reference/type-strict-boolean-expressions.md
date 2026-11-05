---
title: "type/strict-boolean-expressions"
description: "Require safe boolean expressions in script and template conditions."
---

# `type/strict-boolean-expressions`

Require safe boolean expressions in script and template conditions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

`if (count)` relies on the truthiness of a nullable numeric binding instead of an explicit boolean test; it also conflates zero with absence.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## Good

`count !== undefined && count > 0` separately tests presence and positivity, producing an explicit boolean condition after narrowing the optional value.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
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
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7)
