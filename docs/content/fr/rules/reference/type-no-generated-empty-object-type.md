---
title: "type/no-generated-empty-object-type"
description: "Interdire un type d’objet vide produit par une opération de type."
---

# `type/no-generated-empty-object-type`

Interdire un type d’objet vide produit par une opération de type.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Omit retire la seule clé, donc l’alias devient un type d’objet vide.

```vue annotate="remove:2"
<script setup lang="ts">
type Emptied = Omit<{ a: number }, "a">;
</script>
```

## Bon

Omit retire a et laisse b, donc l’alias n’est pas un type d’objet vide.

```vue annotate="add:2"
<script setup lang="ts">
type Kept = Omit<{ a: number; b: string }, "a">;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-generated-empty-object-type": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_generated_empty_object_type.rs#L5)
