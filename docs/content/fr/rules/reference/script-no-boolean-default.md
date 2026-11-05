---
title: "script/no-boolean-default"
description: "Interdire une valeur par défaut sur une prop Boolean."
---

# `script/no-boolean-default`

Interdire une valeur par défaut sur une prop Boolean.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`disabled` et `checked` déclarent tous deux un `default` sur une prop dont le seul constructeur est `Boolean` ; la règle rejette même une valeur par défaut explicite `false`.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // Boolean props already default to false; an explicit default is confusing.
    disabled: { type: Boolean, default: true },
    checked: { type: Boolean, default: false }
  }
}
</script>
```

## Bon

Les props exclusivement booléennes omettent `default` et utilisent la valeur false implicite de Vue. L’union `[Boolean, String]` et la prop Number montrent que ce contrôle se limite au constructeur `Boolean` employé seul.

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
  props: {
    // No explicit default: defaults to false.
    disabled: { type: Boolean },
    disabled2: Boolean,
    // Union type may legitimately need a default.
    value: { type: [Boolean, String], default: '' },
    // Non-Boolean prop.
    count: { type: Number, default: 0 }
  }
}
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
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54)
