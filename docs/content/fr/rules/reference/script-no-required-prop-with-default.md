---
title: "script/no-required-prop-with-default"
description: "Interdire une prop qui possède à la fois required: true et une valeur par défaut."
---

# `script/no-required-prop-with-default`

Interdire une prop qui possède à la fois required: true et une valeur par défaut.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`title` est à la fois obligatoire et doté de la valeur de repli `"Untitled"`, combinant un contrat d’entrée obligatoire avec une valeur par défaut prévue pour une entrée manquante.

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

## Bon

Supprimer `required: true` rend `title` facultatif et conserve `"Untitled"` comme valeur de repli cohérente.

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
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
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L34)
