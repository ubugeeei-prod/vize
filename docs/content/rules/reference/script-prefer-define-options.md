---
title: "script/prefer-define-options"
description: "Prefer defineOptions() over a default export paired with <script setup>."
---

# `script/prefer-define-options`

Prefer defineOptions() over a default export paired with &lt;script setup&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The plain script’s only meaningful statement exports an object containing just `name` and `inheritAttrs`; these options can be expressed by `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## Good

The shown `data()` method makes the script carry real Options API logic, so it falls outside this rule’s conservative options-only suggestion. This Good demonstrates an allowed exception; the direct migration would put `defineOptions({ name: 'MyComponent', inheritAttrs: false })` in `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
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
      "script/prefer-define-options": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L44)
