---
title: "script/no-export-in-script-setup"
description: "Disallow export statements inside <script setup>."
---

# `script/no-export-in-script-setup`

Disallow export statements inside &lt;script setup&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`export const count` attempts to expose a module export from `<script setup>`, where runtime exports are prohibited.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

## Good

Removing `export` keeps `count` as a setup binding rather than a module export.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49)
