---
title: "script/require-define-component"
description: "Require defineComponent() for TypeScript SFC default exports."
---

# `script/require-define-component`

Require defineComponent() for TypeScript SFC default exports.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The TypeScript SFC exports a plain Options API object, so its component options do not receive the contextual types supplied by `defineComponent()`.

```vue annotate="remove:2,7"
<script lang="ts">
export default {
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
}
</script>
```

## Good

Wrapping the object with `defineComponent()` gives props, emits, `this`, and the other component options Vue-aware TypeScript checking.

```vue annotate="add:2,3,8"
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
})
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
        "script/require-define-component": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_define_component.rs#L48)
