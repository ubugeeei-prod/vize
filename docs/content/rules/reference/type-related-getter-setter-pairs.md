---
title: "type/related-getter-setter-pairs"
description: "Require paired getters and setters to use compatible types."
---

# `type/related-getter-setter-pairs`

Require paired getters and setters to use compatible types.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The getter returns string while the setter accepts number, so the pair is incompatible.

```vue annotate="remove:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: number);
}
</script>
```

## Good

The getter and the setter both use string.

```vue annotate="add:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: string);
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
      "type/related-getter-setter-pairs": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/related_getter_setter_pairs.rs#L5)
