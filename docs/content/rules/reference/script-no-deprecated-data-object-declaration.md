---
title: "script/no-deprecated-data-object-declaration"
description: "Disallow an object literal as the component data option (Vue 3 requires a function)."
---

# `script/no-deprecated-data-object-declaration`

Disallow an object literal as the component data option (Vue 3 requires a function).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The Options API `data` option is an object literal, a Vue 2 form that Vue 3 no longer accepts.

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
  // `data` must be a function in Vue 3, not an object literal.
  data: {
    count: 0
  }
}
</script>
```

## Good

`data()` returns a new `{ count: 0 }` object, providing the function-based data declaration required by Vue 3.

```vue annotate="add:3,4"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
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
      "script/no-deprecated-data-object-declaration": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L49)
