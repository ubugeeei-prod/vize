---
title: "type/no-unsafe-member-access"
description: "Interdire l’accès à un membre d’une valeur typée any."
---

# `type/no-unsafe-member-access`

Interdire l’accès à un membre d’une valeur typée any.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

anyValue est any, donc lire member est un accès de propriété non vérifié.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
console.log(anyValue.member);
</script>
```

## Bon

box a une propriété member connue, donc l’accès est vérifié.

```vue annotate="add:2,3"
<script setup lang="ts">
const box = { member: 1 };
console.log(box.member);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-member-access": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_member_access.rs#L5)
