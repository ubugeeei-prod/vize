---
title: "script/no-get-current-instance"
description: "Disallow getCurrentInstance() in Vapor mode (returns null)."
---

# `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | No | None |

Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts.

## Bad

The Vapor-marked setup imports and calls `getCurrentInstance`, relying on an instance API this rule disallows for Vapor-oriented components.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## Good

`inject("app-config")` obtains the explicitly provided configuration without importing or calling `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
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
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38)
