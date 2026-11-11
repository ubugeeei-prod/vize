---
title: "type/no-unnecessary-boolean-literal-compare"
description: "Interdire une comparaison inutile avec un littéral booléen."
---

# `type/no-unnecessary-boolean-literal-compare`

Interdire une comparaison inutile avec un littéral booléen.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

flag est déjà booléen, donc le comparer à true n’ajoute aucun test.

```vue annotate="remove:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag === true;
}
</script>
```

## Bon

Renvoyer flag utilise directement la valeur booléenne.

```vue annotate="add:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag;
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
      "type/no-unnecessary-boolean-literal-compare": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_boolean_literal_compare.rs#L5)
