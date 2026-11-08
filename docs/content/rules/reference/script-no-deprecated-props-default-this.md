---
title: "script/no-deprecated-props-default-this"
description: "Disallow `this` inside a prop default/validator function (removed in Vue 3)."
---

# `script/no-deprecated-props-default-this`

Disallow `this` inside a prop default/validator function (removed in Vue 3).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The prop default and validator read `this`, but those functions cannot rely on the component instance in Vue 3.

```vue annotate="remove:6,7,8"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // `this` is not the component instance in Vue 3.
      default() {
        return this.defaultSize
      }
    },
  }
}
</script>
```

## Good

The default reads `props.baseSize` from its argument, and the validator tests its `value` argument. Both stop depending on an unavailable instance receiver.

```vue annotate="add:6,7,8,11,12,13,14,15,16"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // Vue 3 passes the raw props as the first argument instead.
      default(props) {
        return props.baseSize
      }
    },
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
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
      "script/no-deprecated-props-default-this": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L66)
