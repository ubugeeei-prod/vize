---
title: Configuration
---

<!-- Generated translation; source: guide/configuration.md -->

# Configuration

Vize utilise `vize.config.*` pour les commandes de package npm partagées, le plugin Vite et les paramètres de la ligne de commande Rust.

## Fichiers de configuration

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

## Options du compilateur

Ces options sont placées sous `compiler`. Ils sont soutenus par un schéma et partagés via `defineConfig`; Pas
chaque intégration consomme tous les domaines pour l’instant.

| Option              | Valeurs                               | Usage courant                                                                                               |
| ------------------- | ------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `sourceMap`         | `boolean`                             | Activer les cartes sources dans le plugin Vite                                                              |
| `ssr`               | `boolean`                             | Compiler pour SSR lorsque vous ne dépendez pas du drapeau de compilation SSR de Vite                        |
| `vapor`             | `boolean`                             | Activer la compilation en mode Vapor                                                                        |
| `jsxMode`           | `"vdom"` ou `"vapor"`                 | Backend de sortie par défaut pour les composants `.jsx`/`.tsx`                                              |
| `customRenderer`    | `boolean`                             | Considérez les balises minuscules non HTML comme des éléments de rendu personnalisés                        |
| `customElements`    | `string[]`                            | Motifs de balises compilés comme éléments personnalisés (`Tres*` pour TresJS)                               |
| `templateSyntax`    | `"standard"`, `"strict"`ou `"quirks"` | Choisissez la gestion des avertissements, des erreurs ou des particularités Vue pour la syntaxe des modèles |
| `scriptExt`         | `"ts"` ou `"js"`                      | Conserver la sortie TS ou décompiler vers JS dans la commande de compilation npm                            |
| `mode`              | `"module"` ou `"function"`            | Mode de sortie de compilateur de bas niveau                                                                 |
| `prefixIdentifiers` | `boolean`                             | Identifiants de modèles préfixes avec `_ctx`                                                                |
| `hoistStatic`       | `boolean`                             | Contrôle du levage statique du nœud                                                                         |
| `cacheHandlers`     | `boolean`                             | Mise en cache du gestionnaire d’événements de contrôle                                                      |
| `isTs`              | `boolean`                             | Analyser les blocs de script sous forme de TypeScript                                                       |
| `runtimeModuleName` | `string`                              | Module d’importation à l’exécution de surcharge                                                             |
| `runtimeGlobalName` | `string`                              | Surpasser globalement l’exécution pour la sortie fonction/IIFE                                              |

Pour les projets Vite, les options de plugin direct suppriment la configuration partagée :

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

## Syntaxe des modèles

`compiler.templateSyntax` par défaut sur `"standard"`.

- `"standard"` accepte une syntaxe invalide récupérable, émet des avertissements et réécrit en sortie valide.
- `"strict"` rapporte une syntaxe invalide comme étant des erreurs de compilation.
- `"quirks"` préserve les particularités de compatibilité syntaxique des modèles sans avertissements supplémentaires.

Les cas connus sont :

- `v-for` alias avec une parenthèse d’arête non appariée. Vue dégage une `(` ou une `)` de traînée
  de l’alias précédent il se sépare `value`, `key`et `index`; les modes standard et strict rapportent
  ces alias comme malformés, tandis que le mode quirk reflète Vue.
- Éléments HTML non nul écrits avec une syntaxe auto-fermeuse, tels que `<div />` ou `<span />`.
  mode Standard les avertit et les réécrit comme des éléments vides, des erreurs strictes de mode, et le mode quirk les
  comme des feuilles qui se ferment automatiquement.

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Implémentation en amont de Vue :

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` in `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

Voir [Troubleshooting](./troubleshooting.md) pour le comportement HTML en mode strict derrière les balises invalides
auto-fermantes.

## Mode de sortie JSX & TSX

> Pour l’API complète d’auteur, les styles à cadrage, la vérification de type, le support de l’éditeur et les limitations, voir le
> [JSX & TSX guide](./jsx.md). Cette section ne couvre que les clés de configuration en mode de sortie.

Vize compile les composants Vue `.jsx`/`.tsx` en sortie soit en DOM Virtual, soit en sortie
[Vapor](https://blog.vuejs.org/posts/vue-vapor). `compiler.jsxMode` sélectionne le \*\*global

- - par défaut pour les composants qui ne s’inscrivent pas explicitement ; par défaut, il est `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` est indépendant de `compiler.vapor`: `vapor` bascule Vapor pour `.vue` SFC, tandis que `jsxMode`
contrôle le backend par défaut pour JSX/TSX. Un projet peut garder les SFC sur VDOM tout en mettant par défaut JSX sur
Vapor, ou inversement. Le plugin Vite accepte aussi `jsxMode` directement comme option plugin, ce qui
remplace la configuration partagée.

### Directives par composant

Un composant individuel remplace le défaut par un prologue directif, reflétant `"use strict"`:

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

Comme chaque composant est routé indépendamment, un **seul module peut mélanger les deux backends** :

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### Préséance

Le mode de sortie d’un composant se résout dans cet ordre :

1. Une directive `"use vue:vapor"` / `"use vue:vdom"` par composant.
2. Le `compiler.jsxMode` par défaut depuis la configuration (ou l’option `jsxMode` du plugin).
3. Le plan B intégré, `"vdom"`.

### Diagnostic

Une directive qui commence par `"use vue:"` mais ne nomme pas un mode connu (une faute de frappe comme
`"use vue:vdomx"`) est signalée comme une erreur de compilation plutôt qu’ignorée silencieusement, et deux directives de mode
conflictuelles dans un composant (`"use vue:vapor"` suivies de `"use vue:vdom"`) sont également
diagnostiquées. Des prologues sans lien comme `"use strict"` sont laissés intacts.

## Dialecte Vue

`dialect` sélectionne le profil dialectal Vue pour les documents HTML autonomes (`.html`/`.htm`) :

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"` considère les documents HTML autonomes comme de simples documents Vue-from-CDN.
- `"petite-vue"` opte pour intégrer des documents HTML autonomes dans le
  [petite-vue](https://github.com/vuejs/petite-vue) dialecte (complétions`v-scope`/`v-effect`
  et fonctionnalités IDE sensibles à la petite vue).

Lorsque la clé est absente, le dialecte est détecté structurellement par document : un `<script src>`
résolvant vers le package petite-vue, une importation ES en ligne de `petite-vue`, ou un appel `PetiteVue.createApp`
. Les mentions de petite-vue dans les commentaires ou la prose ne changent jamais de dialecte, et les composantes de
en file indienne utilisent toujours le dialecte standard de Vue.

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
à l’exécution est `@typescript/native-preview`, la couche API Corsa/corsa-bind, et l’exécutable `tsgo`
installé. Gardez les déclarations d’ambiance, les fichiers d’auto-importation générés, les alias de chemin et les déclarations Vue
`ComponentCustomProperties` dans votre `tsconfig.json`de projet, et utilisez un script de paquet
comme `vize:check:app` pour `--tsconfig` ou `--corsa-path` overrides.

```json
{
  "typeChecker": {
    "corsaPath": "./node_modules/.bin/tsgo",
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
