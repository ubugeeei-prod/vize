---
title: "vue/no-child-content"
description: "Interdire le contenu enfant lors de l’utilisation de v-html ou v-text."
---

# `vue/no-child-content`

Interdire le contenu enfant lors de l’utilisation de v-html ou v-text.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

v-text remplace le contenu du paragraphe ; le texte de repli écrit dans le template ne peut donc pas être conservé avec cette directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## Bon

Supprimer le texte enfant laisse v-text comme unique source du contenu du paragraphe.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-child-content": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L32)
