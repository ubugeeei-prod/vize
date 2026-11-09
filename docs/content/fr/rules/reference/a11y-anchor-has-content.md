---
title: "a11y/anchor-has-content"
description: "Exiger un contenu accessible pour les éléments de lien."
---

# `a11y/anchor-has-content`

Exiger un contenu accessible pour les éléments de lien.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Listez les composants dont le sous-arbre rendu fournit toujours un contenu de lien accessible. Les écritures PascalCase et kebab-case sont équivalentes. Les options n'activent pas la règle ; les couches suivantes remplacent la liste entière et [] efface les noms hérités.

## Mauvais

Le lien `/app` n’a ni texte ni autre contenu servant à le nommer ; sa destination n’a donc aucune description accessible.

```vue annotate="remove:2"
<template>
<a href="/app"></a>
</template>
```

## Bon

Le texte `alt` de l’image native fournit un contenu accessible au lien vers la même destination.

```vue annotate="add:2"
<template>
<a href="/app"><img src="/badge.png" alt="Get the app" /></a>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/anchor-has-content": "warn"
    },
    "ruleOptions": {
      "a11y/anchor-has-content": {
        "accessibleChildren": [
          "RetryImage"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L24)
