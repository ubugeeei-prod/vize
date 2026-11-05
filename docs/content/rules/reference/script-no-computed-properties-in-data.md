---
title: "script/no-computed-properties-in-data"
description: "Disallow using computed properties inside the component data function."
---

# `script/no-computed-properties-in-data`

Disallow using computed properties inside the component data function.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`data()` returns `this.doubled` while `doubled` is still a computed property, so the read happens before that computed value exists.

```vue annotate="remove:4,7"
<script lang="ts">
export default {
  data() {
    return { value: this.doubled }
  },
  computed: {
    doubled() { return 1 }
  }
}
</script>
```

## Good

`data()` returns a plain `value`, and `doubled` reads `this.value` only after initialization.

```vue annotate="add:4,7"
<script lang="ts">
export default {
  data() {
    return { value: 1 }
  },
  computed: {
    doubled() { return this.value * 2 }
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
        "script/no-computed-properties-in-data": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_computed_properties_in_data.rs#L44)
