---
title: "script/no-restricted-members"
description: "Disallow project-configured object.property member accesses."
---

# `script/no-restricted-members`

Disallow project-configured object.property member accesses.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | [Rule options](../options.md) |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.

## Bad

With `{ object: "window", property: "localStorage" }` configured in `ruleOptions`, `window.localStorage` accesses the forbidden object/member pair. This rule has no default forbidden members.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## Good

`authStorage.read("token")` delegates the read to the application’s storage helper and no longer accesses the configured `window.localStorage` member.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
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
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53)
