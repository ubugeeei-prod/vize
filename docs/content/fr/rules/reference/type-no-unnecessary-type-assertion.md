---
title: "type/no-unnecessary-type-assertion"
description: "Interdire une assertion de type qui ne change pas le type de l’expression."
---

# `type/no-unnecessary-type-assertion`

Interdire une assertion de type qui ne change pas le type de l’expression.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

known est déjà number, donc l’assertion number ne change pas son type.

```vue annotate="remove:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known as number;
</script>
```

## Bon

Utiliser known sans assertion conserve le type number annoté.

```vue annotate="add:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-assertion": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_assertion.rs#L5)
