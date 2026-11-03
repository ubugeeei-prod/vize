---
title: "ecosystem/vue-router-prefer-named-push"
description: "Prefer named route objects for Vue Router programmatic navigation."
---

# `ecosystem/vue-router-prefer-named-push`

Prefer named route objects for Vue Router programmatic navigation.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

router.push receives a path string that is tied to the current URL spelling.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

## Good

router.push receives a route object with the stable settings name.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
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
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19)
