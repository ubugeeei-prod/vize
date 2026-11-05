---
title: "a11y/no-access-key"
description: "Disallow the use of the accesskey attribute."
---

# `a11y/no-access-key`

Disallow the use of the accesskey attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `accesskey="s"` shortcut may conflict with browser or assistive-technology shortcuts.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

## Good

Removing `accesskey` keeps the ordinary Save button available.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19)
