---
title: "type/require-typed-props"
description: "Exiger une définition de type pour defineProps."
---

# `type/require-typed-props`

Exiger une définition de type pour defineProps.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La déclaration uniquement sous forme de tableau `defineProps(["title"])` déclare `title` par son nom sans lui donner de type.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

## Bon

`defineProps<{ title: string }>()` donne à `title` un type chaîne explicite au lieu d’une déclaration à l’exécution limitée à son nom.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-typed-props": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57)
