---
title: "nuxt/nuxt-config-keys-order"
description: "Préférer l’ordre recommandé des propriétés de configuration Nuxt."
---

# `nuxt/nuxt-config-keys-order`

Préférer l’ordre recommandé des propriétés de configuration Nuxt.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | Oui | Aucune |

Champ d’application : Fichiers de configuration Nuxt (nuxt.config.ts).

## Mauvais

La configuration place `ssr` avant `modules`, inversant leur ordre dans la séquence de clés Nuxt recommandée par la règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## Bon

Placer `modules` avant `ssr` préserve les deux valeurs tout en respectant l’ordre prescrit ; la correction change la disposition plutôt que le sens des options.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24)
