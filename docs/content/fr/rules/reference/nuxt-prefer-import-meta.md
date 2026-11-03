---
title: "nuxt/prefer-import-meta"
description: "Préférer `import.meta.*` à `process.*`."
---

# `nuxt/prefer-import-meta`

Préférer `import.meta.*` à `process.*`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `nuxt` | Oui | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`process.client` utilise un ancien indicateur d’environnement Nuxt que la règle demande de migrer vers `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## Bon

`import.meta.client` conserve la branche réservée au navigateur de manière explicite avec l’indicateur d’environnement de remplacement.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
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
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18)
