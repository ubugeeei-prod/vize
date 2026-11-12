---
title: "type/restrict-plus-operands"
description: "Exiger que les opérandes de + soient explicitement compatibles."
---

# `type/restrict-plus-operands`

Exiger que les opérandes de + soient explicitement compatibles.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

label est une chaîne et numbers est un tableau, donc les deux opérandes de + ne sont pas des primitives compatibles.

```vue annotate="remove:3,4"
<script setup lang="ts">
const label = "a";
const numbers = [1, 2, 3];
const mixedSum = label + numbers;
</script>
```

## Bon

label et extra sont deux chaînes, donc les opérandes de + correspondent.

```vue annotate="add:3,4"
<script setup lang="ts">
const label = "a";
const extra = "b";
const mixedSum = label + extra;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-plus-operands": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_plus_operands.rs#L5)
