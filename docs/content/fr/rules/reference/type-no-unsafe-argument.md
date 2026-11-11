---
title: "type/no-unsafe-argument"
description: "Interdire de passer une valeur de type `any` à une fonction."
---

# `type/no-unsafe-argument`

Interdire de passer une valeur de type `any` à une fonction.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

anyValue est any et il est passé à un paramètre qui attend number.

```vue annotate="remove:2,6"
<script setup lang="ts">
declare const anyValue: any;
function takesNumber(value: number): number {
  return value;
}
takesNumber(anyValue);
</script>
```

## Bon

L’appel passe le nombre 1 au paramètre number.

```vue annotate="add:5"
<script setup lang="ts">
function takesNumber(value: number): number {
  return value;
}
takesNumber(1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-argument": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_argument.rs#L5)
