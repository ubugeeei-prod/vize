---
title: "script/require-slots-as-functions"
description: "Require $slots properties to be called as functions."
---

# `script/require-slots-as-functions`

Require $slots properties to be called as functions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`render` returns `this.$slots.default` without calling it, which was the Vue 2 vnode array.

```vue annotate="remove:4"
<script lang="ts">
export default {
  render() {
    return [this.$slots.default]
  }
}
</script>
```

## Good

`render` calls `this.$slots.default()` and uses the returned vnode.

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
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
      "script/require-slots-as-functions": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L43)
