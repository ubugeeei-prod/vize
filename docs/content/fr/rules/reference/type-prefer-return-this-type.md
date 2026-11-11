---
title: "type/prefer-return-this-type"
description: "Exiger le type `this` lorsqu’une méthode ne renvoie que `this`."
---

# `type/prefer-return-this-type`

Exiger le type `this` lorsqu’une méthode ne renvoie que `this`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

setValue renvoie this, mais le type de retour déclaré est le nom de classe Builder.

```vue annotate="remove:3"
<script setup lang="ts">
class Builder {
  setValue(): Builder {
    return this;
  }
}
</script>
```

## Bon

Le type de retour this correspond à la valeur renvoyée par la méthode.

```vue annotate="add:3"
<script setup lang="ts">
class Builder {
  setValue(): this {
    return this;
  }
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
      "type/prefer-return-this-type": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_return_this_type.rs#L5)
