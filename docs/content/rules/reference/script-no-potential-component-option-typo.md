---
title: "script/no-potential-component-option-typo"
description: "Flag likely typos in Options API component option names."
---

# `script/no-potential-component-option-typo`

Flag likely typos in Options API component option names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The option is spelled `method`, one edit away from the recognized `methods` option; Vue would not treat it as the intended methods declaration.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## Good

Changing the key to `methods` places `save()` under the recognized component option.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-potential-component-option-typo": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19)
