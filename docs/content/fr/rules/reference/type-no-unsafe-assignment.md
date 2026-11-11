---
title: "type/no-unsafe-assignment"
description: "Interdire d’affecter une valeur any à une cible plus précise."
---

# `type/no-unsafe-assignment`

Interdire d’affecter une valeur any à une cible plus précise.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

anyValue est any et il est affecté à une liaison annotée number.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
const assigned: number = anyValue;
</script>
```

## Bon

La liaison est initialisée avec le nombre 1.

```vue annotate="add:2"
<script setup lang="ts">
const assigned: number = 1;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-assignment": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_assignment.rs#L5)
