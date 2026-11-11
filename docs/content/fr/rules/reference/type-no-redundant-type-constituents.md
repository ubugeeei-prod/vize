---
title: "type/no-redundant-type-constituents"
description: "Interdire un membre d’union ou d’intersection qui n’apporte rien ou écrase l’information de type."
---

# `type/no-redundant-type-constituents`

Interdire un membre d’union ou d’intersection qui n’apporte rien ou écrase l’information de type.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

unknown couvre déjà string, donc le constituant string ne restreint pas l’union.

```vue annotate="remove:2"
<script setup lang="ts">
type Redundant = unknown | string;
</script>
```

## Bon

string et number sont tous deux des constituants utiles de l’union.

```vue annotate="add:2"
<script setup lang="ts">
type Redundant = string | number;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-redundant-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_redundant_type_constituents.rs#L5)
