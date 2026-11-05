---
title: "vue/no-unsandboxed-iframe"
description: "Exiger un attribut sandbox sur les éléments iframe."
---

# `vue/no-unsandboxed-iframe`

Exiger un attribut sandbox sur les éléments iframe.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le cadre intégré n’a pas d’attribut sandbox limitant ses capacités.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

## Bon

sandbox applique des restrictions ; allow-scripts autorise explicitement cette seule capacité lorsque cela est nécessaire.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32)
