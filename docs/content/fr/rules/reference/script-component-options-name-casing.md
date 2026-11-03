---
title: "script/component-options-name-casing"
description: "Imposer PascalCase à l’option `name` du composant."
---

# `script/component-options-name-casing`

Imposer PascalCase à l’option `name` du composant.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’option de composant `name: 'my-component'` utilise kebab-case, alors que cette règle exige un nom de composant littéral en PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## Bon

`MyComponent` commence par une majuscule et ne contient que des caractères alphanumériques, ce qui satisfait le contrôle du nom.

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'MyComponent'
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
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44)
