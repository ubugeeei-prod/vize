---
title: "type/prefer-return-this-type"
description: "Enforce that `this` is used when only `this` type is returned."
---

# `type/prefer-return-this-type`

Enforce that `this` is used when only `this` type is returned.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

setValue returns this, but the declared return type is the class name Builder.

```vue annotate="remove:3"
<script setup lang="ts">
class Builder {
  setValue(): Builder {
    return this;
  }
}
</script>
```

## Good

The return type this matches the value that the method returns.

```vue annotate="add:3"
<script setup lang="ts">
class Builder {
  setValue(): this {
    return this;
  }
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
      "type/prefer-return-this-type": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_return_this_type.rs#L5)
