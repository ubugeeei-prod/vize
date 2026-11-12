---
title: "type/no-duplicate-type-constituents"
description: "Interdire un constituant dupliqué dans une union ou une intersection."
---

# `type/no-duplicate-type-constituents`

Interdire un constituant dupliqué dans une union ou une intersection.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

string apparaît deux fois dans la même union, donc le second constituant n’ajoute rien.

```vue annotate="remove:2"
<script setup lang="ts">
type Dup = string | string;
</script>
```

## Bon

L’alias ne conserve qu’un seul constituant string.

```vue annotate="add:2"
<script setup lang="ts">
type Dup = string;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-duplicate-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_duplicate_type_constituents.rs#L5)
