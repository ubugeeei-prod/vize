---
title: "type/require-typed-emits"
description: "Exiger une définition de type pour defineEmits."
---

# `type/require-typed-emits`

Exiger une définition de type pour defineEmits.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La déclaration uniquement sous forme de tableau `defineEmits(["save"])` déclare le nom de l’événement sans contrat typé pour ses arguments.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

## Bon

`defineEmits<{ save: [] }>()` déclare l’événement typé `save` avec un tuple d’arguments vide, indiquant explicitement qu’il n’accepte aucun argument.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54)
