---
title: "type/no-unnecessary-template-expression"
description: "Interdire un littéral de gabarit qui n’enveloppe qu’une expression."
---

# `type/no-unnecessary-template-expression`

Interdire un littéral de gabarit qui n’enveloppe qu’une expression.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Le gabarit ne contient que label, donc il ne construit pas une nouvelle chaîne.

```vue annotate="remove:3"
<script setup lang="ts">
const label = "a";
const wrapped = `${label}`;
</script>
```

## Bon

Affecter label utilise la chaîne sans enveloppe de gabarit.

```vue annotate="add:3"
<script setup lang="ts">
const label = "a";
const wrapped = label;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-template-expression": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_template_expression.rs#L5)
