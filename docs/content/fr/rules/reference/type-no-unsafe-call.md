---
title: "type/no-unsafe-call"
description: "Interdire d’appeler une valeur typée any."
---

# `type/no-unsafe-call`

Interdire d’appeler une valeur typée any.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

anyValue a le type any, donc l’appel ne vérifie pas la cible.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
anyValue();
</script>
```

## Bon

plainNumber est une fonction, donc l’appel a une cible connue.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
plainNumber();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-call": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_call.rs#L5)
