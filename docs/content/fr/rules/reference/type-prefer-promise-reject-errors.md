---
title: "type/prefer-promise-reject-errors"
description: "Exiger qu’une raison de rejet de Promise soit une Error."
---

# `type/prefer-promise-reject-errors`

Exiger qu’une raison de rejet de Promise soit une Error.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Promise.reject reçoit une chaîne, qui n’est pas une raison Error.

```vue annotate="remove:2"
<script setup lang="ts">
const rejected = Promise.reject("failure");
</script>
```

## Bon

Promise.reject reçoit une Error pour le même texte d’échec.

```vue annotate="add:2"
<script setup lang="ts">
const rejected = Promise.reject(new Error("failure"));
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-promise-reject-errors": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_promise_reject_errors.rs#L5)
