---
title: Intégrations expérimentales de bundlers
---

<!-- Generated translation; source: guide/unplugin.md -->

# Intégrations expérimentales de bundlers

> **⚠️ Expérimental :** `@vizejs/unplugin` et `@vizejs/rspack-plugin` restent instables.
> `@vizejs/vite-plugin` reste aujourd’hui l’intégration de bundler la mieux recommandée et la mieux testée.

Vize propose un paquet [unplugin](https://unplugin.unjs.io/) expérimental pour `rollup`, `webpack`et `esbuild`, ainsi qu’un paquet dédié `Rspack` :

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — `Rspack` seulement

Rspack **évite** intentionnellement de suivre le chemin de débranchement partagé.
Sa chaîne de chargement, son `experiments.css`et son comportement HMR nécessitent une manipulation spécifique à Rspack.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez les paquets :

```bash
vp install @vizejs/unplugin
```

Pour le Rspack :

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## Rollup

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

## Webpack

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

## ESBUILD

```javascript
// build.mjs
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

## Rspack

Utilisez le package dédié `@vizejs/rspack-plugin` au lieu de `@vizejs/unplugin`:

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

export default {
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },
  plugins: [new VizePlugin()],
};
```

Voir le package README pour la surface complète de configuration du Rspack.

## Mises en garde

- Vite reste l’intégration recommandée si vous souhaitez le comportement le plus complet et le mieux testé.
- Les modules CSS et les préprocesseurs de style en dehors de Vite dépendent du pipeline CSS du bundler hôte et sont plus susceptibles de changer.
- Si votre bundler inligne l’exécution Vue au lieu de l’externaliser, assurez-vous que les flags habituels de la fonction de compilation Vue sont configurés pour ce bundler.
- Considérez ces intégrations comme expérimentales et validez-les par rapport à votre propre application avant leur déploiement.
