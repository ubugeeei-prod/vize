---
title: "nuxt/no-nuxt-config-test-key"
description: "Interdire la clé `test` dans la configuration Nuxt."
---

# `nuxt/no-nuxt-config-test-key`

Interdire la clé `test` dans la configuration Nuxt.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | Non | Aucune |

Champ d’application : Fichiers de configuration Nuxt (nuxt.config.ts).

## Mauvais

La configuration Nuxt exportée définit la clé d’identifiant `test` sur le booléen `true`, la forme de configuration obsolète rejetée par cette règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

## Bon

La configuration vide supprime cette propriété booléenne `test`. Cet exemple n’interdit pas un objet de configuration de test.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16)
