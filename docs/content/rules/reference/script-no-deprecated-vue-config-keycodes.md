---
title: "script/no-deprecated-vue-config-keycodes"
description: "Disallow Vue.config.keyCodes, which was removed in Vue 3."
---

# `script/no-deprecated-vue-config-keycodes`

Disallow Vue.config.keyCodes, which was removed in Vue 3.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`Vue.config.keyCodes.enter` assigns a removed key-code alias.

```vue annotate="remove:2"
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## Good

The key code is an ordinary constant, `const enter = 13`.

```vue annotate="add:2"
<script setup lang="ts">
const enter = 13
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
        "script/no-deprecated-vue-config-keycodes": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22)
