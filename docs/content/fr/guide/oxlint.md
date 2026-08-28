---
title: Oxlint Plugin
---

<!-- Generated translation; source: guide/oxlint.md -->

# Oxlint Plugin

`oxlint-plugin-vize` permet à Oxlint d’exécuter des diagnostics Vize Patina via le système de plugins JS d’Oxlint.
Utilisez-le lorsque vous voulez les règles JS et TS native Rust d’Oxlint avec les diagnostics
compatibles Vue de Vize en une seule exécution.

Pour le pipeline natif de lint et de vérification de type en dehors d’Oxlint, voir
[Static Analysis](./static-analysis.md).

> [! IMPORTANT]
> Le package est disponible sur npm, mais l’intégration est encore en phase initiale. Pour un terminal lisible par l’homme
> la sortie, préfère `oxlint-vize -f stylish` tandis que la fidélité de la plage SFC d’origine continue de s’améliorer.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez les paquets :

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` résout la liaison native correspondante de Vize via des dépendances optionnelles, de sorte
que la plupart des utilisateurs n’ont pas besoin de `@vizejs/native` installer séparément.

## Usage de base

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

Si vous utilisez une configuration JS ou TS Oxlint, le package exporte également des cartes de règles prédéfinies :

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

Les exportations prédéfinies disponibles incluent :

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## Commandement recommandé

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize`'est un wrapper fin autour de `oxlint` qui adoucit les cas extrêmes `.vue` sans scripts
tandis que la couverture des plugins JS en amont continue de s’améliorer.

## Décors

Les paramètres sont passés par `settings.vize`:

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "general-recommended",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `locale` contrôle le langage de diagnostic.
- `preset` accepte `"general-recommended"`, `"essential"`, `"ecosystem"`, `"incremental"`, `"opinionated"`ou `"nuxt"`.
- `preset` par défaut est `"general-recommended"`.
- `incremental` exécute uniquement les règles que vous configurez explicitement.
- `helpLevel` accepte `"full"`, `"short"`ou `"none"`.
- `typeAware: true` permet de `vize/type/*` des règles soutenues par Corsa lors des passes partagées de Patina.
- `corsaPath` sélectionne le Corsa ou `tsgo` exécutable pour le linting sensible au type.
- `showHelp` et `settings.patina` sont toujours acceptés pour compatibilité rétroactive.

## Limitations actuelles

- Les `oxlint` bruts peuvent encore manquer certains fichiers `.vue` sans `<script>` ni `<script setup>`. Utilisation
  `oxlint-vize` si votre projet inclut uniquement des SFC à base de modèles.
  - Les plugins JS Oxlint ancrent toujours les plages du script extrait, donc le modèle et le style
    diagnostics ne préservent pas encore les plages SFC originales dans tous les formateurs.
- `stylish` est actuellement le meilleur formateur lisible pour un mélange Oxlint + Vize. JSON et
  autres formats lisibles par machine doivent être considérés comme le meilleur effort pour les positions originales de modèle/style
  .
- Les exportations de règles sensibles au type sont expérimentales. Utilisez une configuration `*WithTypeAware` et définissez
  `settings.vize.typeAware: true` quand vous voulez le pass complet partagé pour exécuter ces règles avec enthousiasme.

## Développement local

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxint' build
```
