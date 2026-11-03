---
title: "script/no-reserved-props"
description: "Disallow reserved names in a component's props declaration."
---

# `script/no-reserved-props`

Disallow reserved names in a component's props declaration.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The object-form `ref` and `$foo`, plus the array-form `key`, are reserved prop names. `ref` and `key` are framework controls, and `$`-prefixed names are rejected.

```vue annotate="remove:4,5,8,9,10,11"
<script lang="ts">
export default {
  props: {
    ref: String,   // reserved
    $foo: Number    // `$`-prefixed names are reserved
  }
}

export default {
  props: ['key']    // reserved (array form)
}
</script>
```

## Good

The ordinary prop names `name` and `refValue` avoid the reserved names in both spelling and prefix.

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    name: String,
    refValue: Number
  }
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
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53)
