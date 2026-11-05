---
title: "vue/match-component-file-name"
description: "Require a component name to match its file name."
---

# `vue/match-component-file-name`

Require a component name to match its file name.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The filename is part of this check. The default comparison converts a declared component name to PascalCase or kebab-case and compares it with the unchanged file stem. Files containing several component definitions are left alone.

## Bad

The declared component name `OtherPanel` does not match the `MyPanel.vue` file stem under the supported casing conversions.

`MyPanel.vue`

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "OtherPanel" });
</script>
```

## Good

Name the component `MyPanel` so the declared name matches the file containing it.

`MyPanel.vue`

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyPanel" });
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
        "vue/match-component-file-name": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/match_component_file_name.rs#L21)
