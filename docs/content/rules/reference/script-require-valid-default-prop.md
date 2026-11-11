---
title: "script/require-valid-default-prop"
description: "Require a prop's default value to be valid for its declared type."
---

# `script/require-valid-default-prop`

Require a prop's default value to be valid for its declared type.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The Number and Boolean props receive mismatched scalar defaults, and the Array and Object props use shared literal values instead of factories.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: '0' },     // string default for Number
    enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
    items: { type: Array, default: [] },        // literal must be a factory
    config: { type: Object, default: {} }       // literal must be a factory
  }
}
</script>
```

## Good

The scalar defaults become `0` and `false`; the array and object defaults become functions returning fresh values. The `[String, Number]` example accepts its string default because it matches one declared type.

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: 0 },
    enabled: { type: Boolean, default: false },
    items: { type: Array, default: () => [] },
    config: { type: Object, default: () => ({}) },
    label: { type: [String, Number], default: '' }
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
      "script/require-valid-default-prop": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L73)
