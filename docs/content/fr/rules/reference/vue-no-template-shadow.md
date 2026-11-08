---
title: "vue/no-template-shadow"
description: "Interdire les noms de variables qui masquent des variables d’une portée externe."
---

# `vue/no-template-shadow`

Interdire les noms de variables qui masquent des variables d’une portée externe.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Le contrôle compare les liaisons `v-for`, `v-slot` (y compris `#name`) et les anciennes liaisons de slots à portée limitée au sein du template. Il signale l’identifiant qui masque une liaison précédente, y compris les déclarations du même élément dans l’ordre des attributs. Les éléments frères ont des portées indépendantes. Les clés de propriétés et les références dans les valeurs par défaut ne sont pas des déclarations, et `v-pre` rend son sous-arbre inerte.

Les noms du script et des propriétés du composant sont hors du champ du contrôle actuel : une liaison isolée du template ne produit pas d’avertissement simplement parce qu’une liaison du script porte le même nom. Les plages de diagnostic couvrent l’identifiant tel qu’il est écrit dans le code source, y compris les graphies échappées et les entités HTML.

## Mauvais

Le v-for interne déclare à nouveau item et masque la liaison item externe dans la boucle imbriquée.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## Bon

La boucle interne déclare child, laissant item disponible pour la ligne externe et child pour la ligne imbriquée.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-shadow": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L35)
