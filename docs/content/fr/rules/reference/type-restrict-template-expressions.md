---
title: "type/restrict-template-expressions"
description: "Exiger que les expressions d’un littéral de gabarit soient compatibles avec une chaîne."
---

# `type/restrict-template-expressions`

Exiger que les expressions d’un littéral de gabarit soient compatibles avec une chaîne.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Le gabarit interpole un tableau, qui n’est pas une expression compatible avec une chaîne.

```vue annotate="remove:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const listText = `${numbers}`;
</script>
```

## Bon

Le gabarit interpole label, qui est une chaîne.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
const listText = `${label}`;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-template-expressions": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_template_expressions.rs#L5)
