---
title: "type/only-throw-error"
description: "Exiger que la valeur lancée soit de type Error."
---

# `type/only-throw-error`

Exiger que la valeur lancée soit de type Error.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

La valeur lancée est une chaîne, qui n’est pas une Error.

```vue annotate="remove:3"
<script setup lang="ts">
function fail(): never {
  throw "failure";
}
</script>
```

## Bon

new Error crée une Error pour le même texte d’échec.

```vue annotate="add:3"
<script setup lang="ts">
function fail(): never {
  throw new Error("failure");
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/only-throw-error": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/only_throw_error.rs#L5)
