---
title: "script/no-restricted-members"
---

# `script/no-restricted-members`

Disallow project-configured object.property member accesses

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](../options.md).

This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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

```sh
vp run lint
```

## Bad

With `{ object: "window", property: "localStorage" }` configured in `ruleOptions`, `window.localStorage` accesses the forbidden object/member pair. This rule has no default forbidden members.

```vue
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## Good

`authStorage.read("token")` delegates the read to the application’s storage helper and no longer accesses the configured `window.localStorage` member.

```vue
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [All rules](../all.md)
