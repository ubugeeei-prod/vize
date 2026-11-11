---
title: "type/return-await"
description: "Exiger un usage cohérent de `return await` selon le contexte et la configuration."
---

# `type/return-await`

Exiger un usage cohérent de `return await` selon le contexte et la configuration.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

Dans un try, renvoyer getNumber() sans await fait perdre le rejet à ce catch.

```vue annotate="remove:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
</script>
```

## Bon

return await getNumber() garde le rejet dans le même try et le même catch.

```vue annotate="add:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return await getNumber();
  } catch {
    return 0;
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
      "type/return-await": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/return_await.rs#L5)
