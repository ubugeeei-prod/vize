---
title: "type/related-getter-setter-pairs"
description: "Exiger que les getter et setter appariés utilisent des types compatibles."
---

# `type/related-getter-setter-pairs`

Exiger que les getter et setter appariés utilisent des types compatibles.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Le getter renvoie string alors que le setter accepte number, donc la paire est incompatible.

```vue annotate="remove:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: number);
}
</script>
```

## Bon

Le getter et le setter utilisent tous deux string.

```vue annotate="add:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: string);
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
      "type/related-getter-setter-pairs": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/related_getter_setter_pairs.rs#L5)
