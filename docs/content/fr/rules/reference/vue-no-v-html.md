---
title: "vue/no-v-html"
description: "Déconseiller v-html pour prévenir les vulnérabilités XSS."
---

# `vue/no-v-html`

Déconseiller v-html pour prévenir les vulnérabilités XSS.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

v-html interprète le contenu comme du HTML plutôt que comme du texte ordinaire.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

## Bon

L’interpolation à doubles accolades affiche le contenu comme du texte échappé au lieu d’injecter du HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-html": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51)
