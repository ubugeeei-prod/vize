---
title: Vite Plugin
---

<!-- Generated translation; source: guide/vite-plugin.md -->

# Vite Plugin

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Testez soigneusement avant d’adopter dans des projets non triviaux.

> **Statut du bundler :** `@vizejs/vite-plugin` est actuellement l’intégration de bundler la plus stable.
> Pour le rollup / webpack / esbuild, utilisez `@vizejs/unplugin`, et pour Rspack, utilisez `@vizejs/rspack-plugin`.
> Ces voies non Vite restent instables et doivent être considérées comme expérimentales.

`@vizejs/vite-plugin` fournit une compilation SFC Vue à vitesse native pour les projets Vite. Il est conçu comme un **remplacement direct** pour `@vitejs/plugin-vue` — vos composants Vue existants fonctionnent sans modification.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez les paquets :

```bash
vp install -D @vizejs/vite-plugin
```

Ajoutez `vize` comme dépendance directe uniquement si votre projet importe des assistants de configuration partagés depuis `"vize"`
ou expose des scripts de paquets tels que `vize:lint` et `vize:check`.

## Usage de base

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

C’est tout. Remplacez `@vitejs/plugin-vue` par `@vizejs/vite-plugin` et votre projet sera compilé via Rust.

## Importations TypeScript Vue

Ajoutez le package de plugins à `compilerOptions.types` pour rendre les importations `.vue` directes résolubles par
TypeScript sans écrire de cale de `env.d.ts` locale :

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

Cela ne nécessite pas d’ajouter `vize` comme dépendance directe du projet.

Pour les projets Vite Plus, gardez le type de client Vite Plus et ajoutez le package de plugins :

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

Pour la plupart des projets, gardez les options directes de plugins petites et mettez des paramètres de compilateur stables dans
`vize.config.ts`.

## Configuration partagée

Le point d’entrée partagé recommandé est `vize`. Un seul fichier `vize.config.*` est lu à la fois par les commandes npm
package et `@vizejs/vite-plugin`.

```bash
vp install -D vize
```

Fichiers de configuration pris en charge :

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Configuration TypeScript :

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

Configuration PKL :

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}
```

Configuration JSON avec schéma :

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  }
}
```

Importer `defineConfig` depuis `@vizejs/vite-plugin` fonctionne toujours pour la rétrocompatibilité, mais `import { defineConfig } from "vize"` est le chemin partagé pour la suite.

Voir [Configuration](./configuration.md) pour la configuration partagée complète.

Les projets Vite Plus d’abord peuvent également maintenir les paramètres uniquement de démarrage en ligne dans `vite.config.ts`:

```ts
import { defineConfig } from "vite-plus";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      config: {
        compiler: {
          sourceMap: true,
          vapor: false,
        },
        vite: {
          scanPatterns: ["src/**/*.vue"],
        },
        musea: {
          include: ["src/**/*.art.vue"],
        },
      },
    }),
  ],
});
```

La configuration en ligne est disponible sur le plugin Vite et le store de plugins partagés lors de l’exécution de Vite Plus.
Utilisez `vize.config.*` pour des réglages qui doivent également être lus par les commandes CLI et LSP.

## Options du compilateur

Les options directes ont été transférées à `vize()` `vize.config.*`de dérogation.
La priorité complète est des options directes des plugins, puis des `config`en ligne, puis `vize.config.*`, puis
valeurs par défaut.

```ts
vize({
  vueVersion: 3,
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  templateSyntax: "standard",
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| Option                 | Où le placer                                            | Description                                                                                                                                 |
| ---------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `vueVersion`           | `vize({ vueVersion })`                                  | Réglez `0.11`, `1`, `2`ou `"legacy"` pour exécuter en mode compatibilité Vue non invasif et laissez la compilation SFC au compilateur hôte. |
| `sourceMap`            | `compiler.sourceMap` ou `vize({ sourceMap })`           | Générez des cartes sources. Par défaut, développement activé, production désactivée.                                                        |
| `ssr`                  | `compiler.ssr` ou `vize({ ssr })`                       | Force la compilation SSR quand le drapeau de compilation SSR de Vite ne suffit pas.                                                         |
| `vapor`                | `compiler.vapor` ou `vize({ vapor })`                   | Compile les modèles via le backend Vapor.                                                                                                   |
| `jsxMode`              | `compiler.jsxMode` ou `vize({ jsxMode })`               | Backend de sortie par défaut (`"vdom"` / `"vapor"`) pour `.jsx`/`.tsx` composants. Les directives `"use vue:*"` par composant l’emportent.  |
| `customRenderer`       | `compiler.customRenderer` ou `vize({ customRenderer })` | Considérez les balises minuscules non HTML comme des éléments de rendu personnalisés. Ne correspond pas aux balises PascalCase telles que `<TresMesh>`. |
| `customElements`       | `compiler.customElements` ou `vize({ customElements })` | Motifs de balises compilés comme éléments personnalisés. Utilisez `["Tres*"]` pour les balises PascalCase TresJS.                            |
| `templateSyntax`       | `compiler.templateSyntax` ou `vize({ templateSyntax })` | Choisissez `"standard"`, `"strict"`ou `"quirks"` gestion de la syntaxe du modèle.                                                           |
| `include`              | `vite.include` ou `vize({ include })`                   | Des fichiers que le plugin devrait compiler.                                                                                                |
| `exclude`              | `vite.exclude` ou `vize({ exclude })`                   | Des fichiers que le plugin devrait ignorer.                                                                                                 |
| `scanPatterns`         | `vite.scanPatterns` ou `vize({ scanPatterns })`         | Des motifs glob utilisés pour la précompilation au démarrage.                                                                               |
| `ignorePatterns`       | `vite.ignorePatterns` ou `vize({ ignorePatterns })`     | Les motifs glob étaient sautés lors de la précompilation au démarrage.                                                                      |
| `configMode`           | `vize({ configMode })`                                  | Utilisez `"root"`, `"auto"`ou `false` pour le chargement de configuration partagé.                                                          |
| `configFile`           | `vize({ configFile })`                                  | Chargez un fichier de configuration spécifique.                                                                                             |
| `config`               | `vize({ config })`                                      | Configuration partagée en ligne pour les paramètres d’exécution de Vite Plus.                                                               |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })`                        | Compile `.vue` fichiers importés depuis `node_modules` à la demande.                                                                        |
| `debug`                | `vize({ debug })`                                       | Imprimer les journaux de débogage des plugins.                                                                                              |

Recettes courantes :

```ts
// Vapor-oriented build
vize({ vapor: true });

// Balises PascalCase TresJS
vize({
  customRenderer: true,
  customElements: ["Tres*", "primitive"],
});

// Existing templates that rely on parser edge cases, such as
// v-for alias edge parens or `<div />` as a self-closing leaf
vize({ templateSyntax: "quirks" });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});

// Legacy Vue / Nuxt 2 Bridge project with an existing host compiler plugin
vize({ vueVersion: 2 });
```

`vueVersion: 0.11`, `1`, `2`et `"legacy"` sont des modes de compatibilité hôte-compilateur. Vize ne compile pas
`.vue` fichiers dans ces modes, n’expose pas la cale API `vite:vue` Vue 3, et n’injecte
pas de drapeaux de fonctionnalités du bundler Vue 3. Gardez le plugin de compilateur Vue existant, `vue-loader`, ou le
propre compilateur de Nuxt 2 configurés normalement.

## Comment ça fonctionne

Le plugin intercepte `.vue` requêtes de fichiers et les compile en utilisant le pipeline Rust-native de Vize via Node.js liaisons NAPI :

1. **Pré-compilation** — À `buildStart`, le plugin découvre tous les fichiers `.vue` et les compile en batch à l’aide de `compileBatch`. Cela déclenche une compilation parallèle basée sur Rayon côté Rust, traitant tous les fichiers sur tous les cœurs CPU simultanément.

2. **Compilation à la demande** — Pendant le développement, si un fichier `.vue` est demandé et n’est pas dans le cache (par exemple, importé dynamiquement), il est compilé à la volée via `compileFile`.

3. **HMR** — Lorsqu’un fichier `.vue` change, seul ce fichier est recompilé. Le plugin détecte si le changement est uniquement de style et applique une mise à jour HMR uniquement de style lorsque c’est possible, évitant ainsi un rerendu complet des composants.

4. **Extraction CSS** — Dans les montages de production, tout le CSS à portée des composants Vue est extrait et fusionné en `assets/vize-components.css`, éliminant ainsi la surcharge d’injection par composant.

### Compilation Pipeline

```
.vue file
  → Armature (Parser)          — Tokenizes and parses the SFC structure
  → Croquis (Semantic Analysis) — Analyzes template expressions and bindings
  → Atelier (Compilation)       — Generates optimized JavaScript output
  → Vitrine (NAPI Binding)      — Delivers the result to Node.js
  → Vite module graph            — Served as a virtual module
```

La même couche d’analyse sémantique est réutilisée par le linting et la vérification de types. Voir
[Static Analysis](./static-analysis.md) pour la partie diagnostic du pipeline.

## Comparaison

| Caractéristiques        | @vitejs/plugin-vue | @vizejs/vite-plugin                    |
| ----------------------- | ------------------ | -------------------------------------- |
| Langue                  | JavaScript         | Rouille (NAPI)                         |
| SFC Compilation         | Oui                | Oui                                    |
| Compilation de modèles  | Oui                | Oui                                    |
| Configuration du script | Oui                | Oui                                    |
| Portée CSS              | Oui                | Oui                                    |
| Soutien SSR             | Oui                | Oui                                    |
| HMR                     | Oui                | Oui (optimisation uniquement de style) |
| Précompilation par lots | Non                | Oui (parallèle via Rayon)              |
| CSS Extraction          | Par composant      | File indienne fusionnée                |
| Mode vapeur             | Expérimental       | Première classe (`vize_atelier_vapor`) |

## Fonctionnalités avancées

### Précompilation par lots

Contrairement à `@vitejs/plugin-vue`, qui compile chaque fichier `.vue` dès la première requête, Vize précompile tous les fichiers de `.vue` découverts au démarrage de la compilation en utilisant la compilation par lots multithread. Cela signifie :

- **Démarrage du serveur de développement** — Tous les composants sont prêts avant le premier chargement de la page
- **Constructions de production** — Parallélisme maximal dès le départ

### Réécriture d’actifs statiques

Le plugin réécrit automatiquement les URL des assets statiques dans les modèles. Par exemple :

```vue
<template>
  <img src="./logo.png" />
</template>
```

L’attribut `src` est transféré à une instruction d’importation, permettant à Vite de traiter l’asset via son pipeline d’actifs (hachage, optimisation, etc.).

### Définir le remplacement

Vite saute normalement `import.meta.*` remplacement pour les modules virtuels (préfixé `\0`). Le plugin de Vize applique manuellement des remplacements de définissage pour s’assurer que les valeurs de `import.meta.env.*` fonctionnent correctement dans les composants compilés de Vue.

### Isolement par environnement

Pour la compatibilité Nuxt, le plugin isole `define` valeurs par environnement Vite (client vs. serveur/SSR). Cela empêche les valeurs de l’environnement côté client de fuir dans la sortie SSR.

## Compatibilité Nuxt

Le plugin expose une cale de compatibilité pour les outils qui sondent l’API de `@vitejs/plugin-vue`(comme Nuxt). Cela signifie que Vize fonctionne avec l’intégration intégrée de Nuxt à Vue sans configuration spéciale :

```ts
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

Voir [Nuxt Integration](../integrations/nuxt.md) pour plus de détails.

## Notes

- Le plugin nécessite `@vizejs/native` pour Node.js liaisons NAPI (installées automatiquement en dépendance)
- La compilation en mode vapeur est disponible via `vize_atelier_vapor` (Vue 3.6+)
- La compilation VDOM utilise `vize_atelier_dom`
- Le plugin prend en charge `virtual:vize-styles` pour importer tout le CSS compilé en module
- `.jsx`/`.tsx` composants Vue sont compilés automatiquement via le même plugin — voir le guide [JSX & TSX](./jsx.md)
- Pour le support expérimental du rollup / webpack / esbuild / rspack, voir [Experimental Bundler Integrations](./unplugin.md)
