---
title: "musea/prefer-design-tokens"
---

# `musea/prefer-design-tokens`

Prefer design token CSS variables over hardcoded primitive values

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: See [typed options and defaults](../options.md).

Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
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

The art example uses the literal blue color instead of the configured primary design token.

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

## Good

The style refers to --color-primary, the token configured for this example.

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [All rules](../all.md)
