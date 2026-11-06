---
title: "script/require-render-return"
description: "Require the component render function to return a value."
---

# `script/require-render-return`

Require the component render function to return a value.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The `render` function reads `this.count` inside a branch and never returns a value.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  render() {
    if (this.ready) {
      this.count
    }
  }
}
</script>
```

## Good

The `render` function returns `this.count`.

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.count
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
      "script/require-render-return": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_render_return.rs#L35)
