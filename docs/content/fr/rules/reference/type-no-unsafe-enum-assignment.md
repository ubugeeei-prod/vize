---
title: "type/no-unsafe-enum-assignment"
description: "Interdire d’affecter une valeur étrangère à un enum."
---

# `type/no-unsafe-enum-assignment`

Interdire d’affecter une valeur étrangère à un enum.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La liaison est annotée Color, mais elle est initialisée avec le nombre 1.

```vue annotate="remove:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
const color: Color = 1;
</script>
```

## Bon

Color.Red est un membre de l’enum Color.

```vue annotate="add:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
const color: Color = Color.Red;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-enum-assignment": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_enum_assignment.rs#L5)
