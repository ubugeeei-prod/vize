---
title: Dépannage
---

<!-- Generated translation; source: guide/troubleshooting.md -->

# Dépannage

## Modes de syntaxe du modèle

Vize `compiler.templateSyntax` par défaut à `"standard"`. Le mode standard accepte les problèmes de syntaxe récupérables
un modèle, rapporte les avertissements et les réécrit en résultats valides.

Un cas courant de migration est la syntaxe auto-fermeture sur des éléments HTML non nuls :

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` et `<span />` ne sont pas des éléments HTML valides auto-fermants. Le mode standard les réécrit comme
éléments vides, équivalents à `<div></div>` et `<span></span>`, et émet un avertissement. Le mode strict
les signale comme des erreurs. Le mode Alter les maintient comme des feuilles qui se ferment automatiquement sans avertissement.

Préfèrent écrire des balises de fin explicites :

```vue
<template>
  <div></div>
  <span></span>
</template>
```

Choisissez explicitement un mode lors de la migration :

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      templateSyntax: "standard",
    }),
  ],
};
```

Utilisez `"strict"` pour échouer sur une syntaxe invalide, ou `"quirks"` lorsqu’un projet dépend que Vue accepte ces balises
comme des laisses auto-fermables. Les éléments du vide valides comme `<input />`, `<img />`, `<br />`
`<meta />` n’ont pas besoin d’alters.

## Résolution native des paquets de types

`vize check` résout les paquets de type Vue et Vite du projet vérifié avant d’utiliser des solutions de secours
groupées, donc les propres versions `vue`, `@vue/runtime-dom`, `@vue`et `vite` du projet pilotent le projet virtuel généré
. Pour les dispositions inhabituelles du gestionnaire de paquets, définissez `VIZE_VUE_PACKAGE`,
`VIZE_VUE_NAMESPACE_PACKAGE`, `VIZE_VUE_RUNTIME_DOM_PACKAGE`ou `VIZE_VITE_PACKAGE` à des racines explicites
paquets. `VIZE_RUNTIME_NODE_MODULES` peut aussi pointer vers une ou plusieurs racines `node_modules` comme chemin de recherche
de rechange.
