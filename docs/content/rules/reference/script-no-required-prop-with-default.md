---
title: "script/no-required-prop-with-default"
description: "Disallow a prop that is both required: true and has a default."
---

# `script/no-required-prop-with-default`

Disallow a prop that is both required: true and has a default.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`title` is both required and given the fallback `"Untitled"`, combining a required-input contract with a default intended for missing input.

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

## Good

Removing `required: true` makes `title` optional and leaves `"Untitled"` as its coherent fallback.

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-required-prop-with-default": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L34)
