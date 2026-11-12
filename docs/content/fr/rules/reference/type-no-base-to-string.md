---
title: "type/no-base-to-string"
description: "Interdire la conversion en chaîne des valeurs qui retombent sur Object.prototype.toString()."
---

# `type/no-base-to-string`

Interdire la conversion en chaîne des valeurs qui retombent sur Object.prototype.toString().

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

plain est un objet simple, donc toString() retombe sur Object.prototype.toString().

```vue annotate="remove:2,3"
<script setup lang="ts">
const plain = { a: 1 };
plain.toString();
</script>
```

## Bon

label est déjà une chaîne, donc toString() utilise la conversion de chaîne.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
label.toString();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-base-to-string": "warn"
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
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_base_to_string.rs#L5)
