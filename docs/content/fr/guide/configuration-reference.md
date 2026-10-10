---
title: Référence de configuration partagée
---

<!-- Reviewed translation; source: guide/configuration-reference.md; scope: introduction and discovery -->

# Référence de configuration partagée

Commencez par les fichiers existants `vite.config.*` et `tsconfig.json` ; consultez le [guide de configuration](./configuration.md). Cette page décrit les réglages natifs et les formats dédiés facultatifs.

## Fichiers de configuration

La configuration Vite est découverte lorsqu'aucune configuration dédiée n'existe dans le même répertoire du projet le plus proche. La CLI accepte aussi `--config` ; les options directes du plugin et les fonctions explicitement choisies dans l'éditeur ont priorité. Les chemins relatifs partent du répertoire de configuration. Utilisez `vize.entries` et le projet TypeScript du paquet pour définir les périmètres.


Le paquet npm commande et `@vizejs/vite-plugin` charger ces fichiers depuis la racine du projet dans cet ordre de priorité
:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

La ligne de commande Rust lit les mêmes noms de fichiers de configuration dans l’ordre ci-dessus pour les paramètres natifs des commandes tels que
`check`, `lint`, `lsp`et `fmt`.

## Configuration TypeScript

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Résolution de type Vue

Vize ne fixe pas la surface de type de Vue à partir du paquet `vize` publié : `vize check`, le langage
le serveur et les commandes package résolvent `vue`, `@vue/compiler-sfc`, et types d’ambiance associés issus du projet
analysé, donc les choix de patch, mineurs et pré-release de Vue 3 restent sous le contrôle de ce projet
plutôt que la version utilisée pour construire Vize. Pour des résultats prévisibles, déclarez la version prise en charge de Vue
dans le projet utilisateur (pas via les internes Vize), gardez `vue`, `@vue/compiler-sfc`
intégrations comme Nuxt alignées à cet endroit, et exécutez `vize check` depuis la racine du projet ou un point
`typeChecker.tsconfig` vers le package cible ; utiliser `typeChecker.corsaPath` uniquement pour choisir le checker
binaire, jamais pour remplacer les versions de type Vue. Lorsqu’un projet prend en charge plusieurs plages Vue, testez chaque
dans sa propre matrice de paquets afin que Vize suive le graphe de dépendance active, et non un chemin de type codé en dur.

## Entrées expérimentales sur le plat

Les monorepos peuvent décrire les paramètres par défaut racines et les overrides à portée de paquet avec `entries`. Les configurations d’objets
simples sont normalisées en une seule entrée en interne, et les exportations de tableaux sont acceptées par `defineConfig` pour
création de type ESLint-flat-config.

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## Configuration PKL

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## Configuration JSON

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## Options d’analyse statique

Utilisez `linter` pour le chemin de peluches npm :

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

Utilisez `typeChecker` pour le chemin de vérification du npm :

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Vue 3 Options API template bindings; default-on (matches vue-tsc).
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` résout les liaisons de modèles API des options de Vue 3
(`data`/`computed`/`methods`/`inject`/`setup`/`props` sur un `<script> export default { ... }`simple).
Il est livré dans la version standard (pas la fonction `legacy`), est **activé par défaut** (correspondant `vue-tsc`),
et ne fonctionne que pour des composants non`<script setup>`, de sorte que le chemin commun reste sans coût ; Configurez
`optionsApi: false` pour vous désinscrire. Le support Legacy Vue 2.7 / Nuxt 2 (`typeChecker.legacyVue2`, qui ajoute
les globals de modèles Nuxt 2) est un op-in séparé `legacy`-build.

`typeChecker.tsconfig` et `typeChecker.corsaPath` font partie du schéma partagé, mais le chemin Corsa
soutenu par le projet est aujourd’hui la surface Rust CLI. `corsaPath` est partagé par `vize check`,
`vize lint`sensibles au type , et `vize lsp` (`typeChecker.tsgoPath` est un alias obsolète) ; la pile
à l’exécution est le package de plateforme native TypeScript 7 (`typescript` / `@typescript/typescript-*`)
avec la couche API Corsa/corsa-bind. Laissez `corsaPath` non défini sauf si vous devez pointer Vize
vers un exécutable `lib/tsc` installé précis. Gardez les déclarations d’ambiance, les fichiers d’auto-importation générés, les alias de chemin et les déclarations Vue
`ComponentCustomProperties` dans votre `tsconfig.json`de projet, et utilisez un script de paquet
comme `vize:check:app` pour `--tsconfig` ou `--corsa-path` overrides.

```json
{
  "typeChecker": {
    "servers": 1
  }
}
```

`typeChecker.servers` est réservé aux futurs pools de travailleurs Corsa. Le runner direct de session de projet
ne supporte actuellement que `1`; les valeurs plus élevées échouent rapidement au lieu de faire semblant d’ajuster la concurrence.

## Musea Options

La configuration partagée couvre actuellement l’ensemble de fichiers et le routage de la galerie :

```ts
export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

Passez directement des options axées sur la présentation telles que `previewCss`, `previewSetup`, `tokensPath`, `theme`et
`storybookOutDir` directement pour `musea()` dans `vite.config.ts`.
