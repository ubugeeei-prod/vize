---
title: "vue/html-self-closing"
description: "Imposer un style de fermeture automatique des balises."
---

# `vue/html-self-closing`

Imposer un style de fermeture automatique des balises.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Oui | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le composant vide utilise une paire de balises, tandis que les éléments vides img et br omettent la syntaxe autofermante configurée.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

## Bon

Le composant et les éléments vides utilisent une syntaxe autofermante ; un div contenant du contenu conserve sa balise de fermeture.

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-self-closing": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L31)
