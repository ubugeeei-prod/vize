---
title: "type/no-unsafe-return"
description: "Interdire de renvoyer une valeur any depuis une fonction."
---

# `type/no-unsafe-return`

Interdire de renvoyer une valeur any depuis une fonction.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La fonction promet un number, mais elle renvoie anyValue, qui est any.

```vue annotate="remove:2,4"
<script setup lang="ts">
declare const anyValue: any;
function returnsAny(): number {
  return anyValue;
}
</script>
```

## Bon

La fonction renvoie le nombre 1, qui correspond au type de retour annoté.

```vue annotate="add:3"
<script setup lang="ts">
function returnsAny(): number {
  return 1;
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
      "type/no-unsafe-return": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_return.rs#L5)
