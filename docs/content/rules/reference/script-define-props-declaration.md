---
title: "script/define-props-declaration"
description: "Enforce type-based defineProps<{ ... }>() over the runtime/object form."
---

# `script/define-props-declaration`

Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`defineProps({ title: String })` supplies a runtime object, which conflicts with this rule’s preference for type-based props.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

## Good

`defineProps<{ title: string }>()` declares `title` in the type argument and retains the `props.title` access without a runtime declaration argument.

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-props-declaration": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40)
