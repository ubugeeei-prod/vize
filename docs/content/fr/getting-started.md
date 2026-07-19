---
title: Commencer
---

<!-- Generated translation; source: getting-started.md -->

# Commencer

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les API et les limites des paquets peuvent changer sans préavis.

## Qu’est-ce que Vize ?

Vize (_/viːz/_) est une chaîne d’outils Vue.js écrite en Rust. L’espace de travail contient des blocs de construction
partagés pour :

| Superficie               | Caisse principale de rouille                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | Point d’entrée orienté utilisateur             |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| Compilation              | [`vize_atelier_core`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_core), [`vize_atelier_dom`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_dom), [`vize_atelier_vapor`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_vapor), [`vize_atelier_ssr`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_ssr), [`vize_atelier_sfc`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`, npm `vize:build` script |
| Peluches                 | [`vize_patina`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                                             | NPM `vize:lint` script, `oxlint-plugin-vize`   |
| Format                   | [`vize_glyph`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                                               | Script `vize:fmt` NPM                          |
| Contrôle de type         | [`vize_canon`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                                               | NPM `vize:check` script                        |
| Support des éditeurs     | [`vize_maestro`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                                           | VS Code, Zed, Rust `vize lsp`                  |
| Outils d’art de la musea | [`vize_musea`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                                               | `@vizejs/vite-plugin-musea`                    |
| Reliures                 | [`vize_vitrine`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                                           | `@vizejs/native`, `@vizejs/wasm`               |

Ce guide recommande [Vite+](https://viteplus.dev/) (`vp`) pour la gestion de paquets JavaScript et les commandes de projet. Cela maintient la cohérence du flux d’installation et d’exécutif entre les gestionnaires de paquets tout en utilisant l’outil sous-jacent de l’espace de travail.

Si vous n’avez pas encore `vp` , installez-le une fois et ouvrez un nouveau shell :

```bash
curl -fsSL https://vite.plus | bash
```

Consultez les [Vite+ docs](https://viteplus.dev/) et les [Installing Dependencies guide](https://viteplus.dev/guide/install) pour en savoir plus.

## Ce que fait Vize

À un niveau général, Vize est divisé en quelques voies réutilisables :

| Voie                  | Paquet ou script                         | Ce que tu obtiens                                                                                           |
| --------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Compiler              | `@vizejs/vite-plugin`, `vize:build`      | Compilation SFC Vue native Rust, sortie SSR, mode Vapor, gestion CSS à portée portée                        |
| Analyse statique      | `vize:lint`, `oxlint-plugin-vize`        | Modèle Vue, script, CSS, a11y, SSR, Vapor, Musea, diagnostics cross-file et sensibles aux types             |
| Contrôle de type      | `vize:check`                             | Génération de Virtual TypeScript, diagnostic de projet, mappage de diagnostic Vue vers la source            |
| Format                | `vize:fmt`                               | Mise en page SFC Vue avec options de projet et de CLI                                                       |
| Galerie de composants | `@vizejs/vite-plugin-musea`, `musea-vrt` | Fichiers artistiques, variantes composantes, configuration de prévisualisation, jetons de design, a11y, VRT |
| Support des éditeurs  | VS Code, Zed, Rust `vize lsp`            | Diagnostics et fonctionnalités d’éditeur en option                                                          |

Voir [Static Analysis](./guide/static-analysis.md) pour le modèle de vérification de lint et de type,
[Rules](./rules/index.md) pour la sortie de règles concrètes, et
[Configuration](./guide/configuration.md) pour les options partagées de configuration et de compilateur.

Créer des composants dans JSX/TSX au lieu de `.vue` SFC ? Consultez le guide [JSX & TSX](./guide/jsx.md) —
`.jsx`/`.tsx` composants Vue se compilent dans la même voie Rust.

## Choisissez votre point d’entrée

### 1. Projets Vite

Utilisez le plugin Vite si vous voulez une compilation native de Vue dans un projet Vite existant.

```bash
vp install -D @vizejs/vite-plugin
```

Installez `vize` comme dépendance directe uniquement lorsque vous souhaitez importer des assistants de configuration partagés depuis
`"vize"` ou ajouter des scripts de paquets Vize comme `vize:lint` et `vize:check`.

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Ajoutez des options de compilateur dans `vize.config.ts` lorsque vous souhaitez avoir les mêmes réglages pour intégrer
scripts et le plugin :

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

### 2. Projets Nuxt

Utilisez le module Nuxt lorsque vous voulez que Vize tourne dans le pipeline Vite de Nuxt.

```bash
vp install @vizejs/nuxt
```

Ajoutez le module à `nuxt.config.ts`:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

Faites tourner votre serveur de développement Nuxt comme d’habitude. Les registres du module `@vizejs/vite-plugin` pour la compilation de
SFC Vue tout en préservant les importations automatiques Nuxt, les composants, les middlewares et les transformations SSR.

Consultez le guide [Nuxt Integration](./integrations/nuxt.md) pour l’installation de Musea et les notes spécifiques à Nuxt.

### 3. scripts de paquet npm + configuration partagée

Utilisez le package `vize` npm lorsque vous voulez des utilitaires de configuration partagés et des commandes natives disponibles via
scripts de projet.

```bash
vp install -D vize
```

Scripts de paquets recommandés :

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:build
vp run vize:ready
```

La commande `vize check` du package npm utilise le vérificateur NAPI empaqueté et peut émettre des déclarations de composantes Vue
avec `--declaration --declaration-dir dist/types`. Utilisez la ligne de commande Rust lorsque vous avez besoin de
`check-server`, LSP, gestion de l’IDE ou de diagnostics de projet via Vue, TS, TSX et `.d.ts` entrées.

### 4. CLI complet de rouille

La plupart des flux de travail applicatifs devraient utiliser les scripts de paquet npm ci-dessus. Utilisez le binaire Rust lorsque vous
besoin de la CLI native complète aujourd’hui : LSP, gestion de l’IDE, profilage ou `check-server`. Pour l’alpha v1, les canaux publics
pris en charge sont les binaires de publication GitHub et le point d’entrée Nix ; le CLI Rust n’est pas encore
publié via crates.io.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize ready src
vize lsp
```

## Vérification de type native

`vize check` est alimenté par `vize_canon`, qui s’appuie désormais sur [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) sessions de projet pour des diagnostics natifs TypeScript. Vize génère un TypeScript virtuel pour les SFC Vue, demande à Corsa des diagnostics conscients du projet, puis remappe les résultats sur les fichiers originaux `.vue`, `.ts`, `.tsx`et `.d.ts`.

Cette voie est encore en train de mûrir, donc la vérification des types d’éditeurs reste une option volontaire pour l’instant. La pile d’exécution
est le paquet `@typescript/native-preview`, Corsa/corsa-bind est la couche API avec laquelle Vize
communique, et l’exécutable installé par l’aperçu natif TypeScript est encore couramment nommé
`tsgo`. Utilisez `typeChecker.corsaPath`, ou un script package qui s’exécute
`vize check --corsa-path /path/to/tsgo`, lorsque vous souhaitez épingler ce runtime.
`typeChecker.tsgoPath` reste un alias de compatibilité obsolète.

Cibles utiles pour scripts de paquets :

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:app
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

## Partagé `vize.config.*`

Les commandes de paquet npm et `@vizejs/vite-plugin` partagent la découverte de configuration :

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Configuration TypeScript :

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    corsaPath: "./node_modules/.bin/tsgo",
  },
  formatter: {
    printWidth: 100,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

Configuration PKL :

```pkl
amends "node_modules/vize/pkl/vize.pkl"

linter {
  preset = "opinionated"
}

typeChecker {
  enabled = true
  strict = true
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

Configuration JSON avec schéma :

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "linter": {
    "preset": "opinionated"
  }
}
```

## Paquets

```bash
vp install -D @vizejs/vite-plugin
vp install @vizejs/native
vp install @vizejs/wasm
vp install @vizejs/unplugin
vp install @vizejs/rspack-plugin @rspack/core
vp install @vizejs/nuxt
vp install @vizejs/vite-plugin-musea
vp install @vizejs/musea-mcp-server
vp install -D oxlint oxlint-plugin-vize
```

Notes :

- `@vizejs/vite-plugin` 'est l’intégration recommandée du bundler aujourd’hui.
- `@vizejs/unplugin` et `@vizejs/rspack-plugin` sont encore expérimentaux.
- `@vizejs/native` et `@vizejs/wasm` exposent directement les fixations Rust.
- `@vizejs/vite-plugin-musea` fournit la galerie et le flux de travail dev-server pour Musea.

## Galerie des composantes de la Musea

Utilisez Musea lorsque vous souhaitez des exemples de composants natifs Vue, de la documentation, des jetons, des vérifications VRT et a11y :

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

Faites tourner votre serveur de développement Vite et ouvrez `/__musea__`. Voir [Musea](./guide/musea.md) pour les fichiers d’art, la configuration
aperçu, les jetons de design, la VRT et les variantes générées.

## Intégration Oxlint

Exécutez les diagnostics Vue de Vize à l’intérieur d’Oxlint :

```bash
vp install -D oxlint oxlint-plugin-vize
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  },
  "settings": {
    "vize": {
      "preset": "general-recommended",
      "helpLevel": "short"
    }
  }
}
```

Pour une utilisation terminale d’abord, préférez :

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

## Support des éditeurs

Pour le montage quotidien de Vue, continuez à utiliser `vuejs/language-tools` pour l’instant.
fonctionnalités de l’éditeur Vize sont conçues pour l’adhésion incrémentale.

Point de départ de VS Code :

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Point de départ de Zed :

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## Développement local

Les tâches locales restent locales ; [CI parity](./contributing.md#common-checks) utilise `nix develop .#testbox`.

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
vp build
```
