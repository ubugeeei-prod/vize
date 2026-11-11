---
title: "type/no-mixed-enums"
description: "Interdire de mélanger des membres chaîne et numériques dans un enum."
---

# `type/no-mixed-enums`

Interdire de mélanger des membres chaîne et numériques dans un enum.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

L’enum mélange un membre numérique et un membre chaîne.

```vue annotate="remove:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = "b",
}
</script>
```

## Bon

Les deux membres sont numériques, donc l’enum a un seul domaine de valeurs.

```vue annotate="add:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = 1,
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-mixed-enums": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_mixed_enums.rs#L5)
