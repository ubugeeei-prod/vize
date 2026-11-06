---
title: "script/no-deprecated-destroyed-lifecycle"
description: "Disallow deprecated destroyed and beforeDestroy lifecycle hooks."
---

# `script/no-deprecated-destroyed-lifecycle`

Disallow deprecated destroyed and beforeDestroy lifecycle hooks.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Yes | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`beforeDestroy` is the removed Vue 2 lifecycle option used for the timer cleanup.

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

## Good

Renaming the hook to `beforeUnmount` preserves the cleanup body under its Vue 3 lifecycle name.

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-destroyed-lifecycle": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17)
