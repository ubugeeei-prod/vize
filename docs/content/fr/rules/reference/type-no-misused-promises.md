---
title: "type/no-misused-promises"
description: "Interdire une Promise là où elle n’est pas prise en charge."
---

# `type/no-misused-promises`

Interdire une Promise là où elle n’est pas prise en charge.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

forEach attend un callback void, mais le callback async renvoie une Promise.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach(async (n) => {
  console.log(n);
});
</script>
```

## Bon

Le callback n’est pas async, donc il renvoie void comme forEach l’attend.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach((n) => {
  console.log(n);
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-promises": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_promises.rs#L5)
