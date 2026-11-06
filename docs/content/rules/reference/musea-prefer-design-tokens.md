---
title: "musea/prefer-design-tokens"
description: "Prefer design token CSS variables over hardcoded primitive values."
---

# `musea/prefer-design-tokens`

Prefer design token CSS variables over hardcoded primitive values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Musea .art.vue art, variant, and style blocks.

Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.

## Bad

The art example uses the literal blue color instead of the configured primary design token.

`Button.art.vue`

```vue annotate="remove:6"
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

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

Then run `vp run lint`.

## Related

- [Musea rules](../musea.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32)
