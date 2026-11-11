---
title: "type/no-unnecessary-type-conversion"
description: "Interdire une conversion qui ne change ni le type ni la valeur de l’expression."
---

# `type/no-unnecessary-type-conversion`

Interdire une conversion qui ne change ni le type ni la valeur de l’expression.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

label est déjà une chaîne, donc String(label) ne change ni son type ni sa valeur.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const same = String(label);
</script>
```

## Bon

count est un nombre, donc String(count) le convertit en chaîne.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const same = String(count);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-conversion": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_conversion.rs#L5)
