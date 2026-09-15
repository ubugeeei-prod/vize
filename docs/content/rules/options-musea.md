---
title: Musea Rule Options
---

# Musea Rule Options

Musea rule option examples cover design-token inventory configuration.. The main [Rule Options](./options.md) page lists the complete option table,
unknown-field behavior, and scoped replacement semantics.

## `musea/prefer-design-tokens`

Use this option to connect Musea/style linting to the project's design-token inventory. The rule is
off unless token data is configured and the rule is enabled or selected by a non-empty token list.
Each token maps a hardcoded CSS value to a CSS custom property derived from `path`.

```json
{
  "linter": {
    "rules": { "musea/prefer-design-tokens": "warn" },
    "ruleOptions": {
      "musea/prefer-design-tokens": {
        "tokens": [
          { "path": "color.primary", "value": "#3b82f6" },
          { "path": "color.danger", "value": "#ef4444", "tier": "semantic" }
        ]
      }
    }
  }
}
```

Bad with the config above:

```vue
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```
