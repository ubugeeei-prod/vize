---
title: Nuxt
description: Ajoutez Vize à un projet Nuxt avec un seul module pour compiler, analyser et prévisualiser les composants Vue.
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Nuxt

`@vizejs/nuxt` ajoute Vize à un projet Nuxt sous la forme d’un seul module. Il compile les fichiers `.vue` avec le compilateur Rust de Vize à la place du compilateur Vue par défaut de Nuxt, génère une configuration de lint adaptée à Nuxt et peut servir la galerie de composants [Musea](../guide/musea.md). Les imports automatiques, les composants, les middlewares et le SSR continuent de fonctionner via Nuxt.

> [!WARNING]
> Vize est en cours de développement et n’est pas prêt pour la production. Comparez la sortie de build et les diagnostics sur votre propre projet avant de l’adopter, et consultez l’[état de la prise en charge](../stability.md).

## Mise en place

### 1. Installer le module

Installez `vp` une fois depuis le [guide d’installation de Vite+](https://viteplus.dev/guide/install), puis ajoutez le module :

```bash
vp install @vizejs/nuxt
```

### 2. Enregistrer le module

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

`compiler: true` est la valeur par défaut : `modules: ["@vizejs/nuxt"]` suffit. Toutes les options de cette page se placent sous la clé `vize`.

### 3. Démarrer Nuxt

Démarrez le serveur de développement comme d’habitude :

```bash
vp run dev
```

Le module ajoute `@vizejs/vite-plugin` à la configuration Vite de Nuxt et conserve les transformations propres à Nuxt. Les pages, les layouts, les composables et les composants importés automatiquement se comportent comme avant.

### 4. Vérifier que Vize compile vos composants

Activez les journaux de débogage du compilateur pour voir les fichiers traités par Vize :

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { debug: true },
  },
});
```

Nuxt compile les composants à la demande : le journal affiche `[vize] load: on-demand compiling ...` uniquement pour les fichiers réellement atteints par une requête. Retirez `debug` une fois la configuration vérifiée.

<span id="options"></span>

## Options

Toutes les options se placent sous `vize` dans `nuxt.config.ts`. La plupart des projets n’ont besoin que des quatre premières.

| Option                | Type                                    | Défaut                     | Rôle                                                                                                                                                                       |
| --------------------- | --------------------------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions`    | `true`                     | Compile les SFC Vue avec Vize. Un objet transmet ses options à `@vizejs/vite-plugin`.                                                                                      |
| `lint`                | `boolean \| VizeNuxtLintOptions`        | `true`                     | Génère la configuration Oxlint adaptée à Nuxt. Voir [Lint](#lint).                                                                                                         |
| `checker`             | `boolean \| VizeNuxtLintCheckerOptions` | `false`                    | Lance le lint à côté du serveur de développement et affiche les résultats dans le terminal et le navigateur. Voir [Lint pendant le développement](#lint-while-developing). |
| `musea`               | `boolean \| MuseaOptions`               | `false`                    | Sert la galerie Musea. Voir [Musea](#musea).                                                                                                                               |
| `ui`                  | `boolean \| VizeNuxtUiOptions`          | `false`                    | Enregistre automatiquement les composants `@vizejs/ui`. Voir [imports automatiques](../guide/auto-imports.md#nuxt).                                                        |
| `composables`         | `boolean \| VizeNuxtComposablesOptions` | `false`                    | Importe automatiquement les fonctions de `@vizejs/composable`. Voir [imports automatiques](../guide/auto-imports.md#nuxt).                                                 |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`      | `true`                     | Réapplique aux modules compilés par Vize les transformations Nuxt : imports automatiques, composants, i18n et clés de données asynchrones.                                 |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`      | `true`                     | Permet à UnoCSS de lire la source `.vue` d’origine. `originalSource: false` le désactive ; `maxBytes` limite la taille lue par fichier.                                    |
| `dev.stylesheetLinks` | `boolean`                               | `true`                     | Nettoie les liens de feuilles de style dupliqués ou dangereux dans le HTML SSR de développement.                                                                           |
| `compatibility`       | `VizeNuxtCompatibilityOptions`          | détectée                   | Remplace les versions majeures de Nuxt et Vue détectées. Voir [Nuxt 2 et Vue 2](#nuxt-2-and-vue-2).                                                                        |
| `nuxtMusea`           | `NuxtMuseaOptions`                      | `{ route: { path: "/" } }` | Décrit la route, la configuration d’exécution et les simulations de fetch visibles dans les aperçus Musea.                                                                 |

### Compilateur

Toute option de `@vizejs/vite-plugin` peut être transmise via `compiler`. Nuxt garde ses propres valeurs par défaut pour la racine du projet, la compilation à la demande et les fichiers `.vue` des dépendances :

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      // Usually inferred automatically.
      // Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
      vueVersion: 3,
    },
    compiler: {
      // Any @vizejs/vite-plugin option can be passed here.
      configMode: "auto",
      customRenderer: false,
      debug: false,
      handleNodeModulesVue: true,
      ignorePatterns: ["node_modules/**", ".nuxt/**", ".output/**"],
      precompileBatchSize: 64,
      scanPatterns: [], // Nuxt defaults to on-demand compilation
      sourceMap: true,
      vapor: false,
    },
    bridge: {
      autoImports: true,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    unocss: {
      originalSource: {
        maxBytes: 2 * 1024 * 1024,
      },
    },
    dev: {
      stylesheetLinks: true,
    },
    musea: false,
  },
});
```

Le [guide du plugin Vite](../guide/vite-plugin.md#compiler-options) décrit chaque option du compilateur. Avec `compiler: false`, Nuxt garde son compilateur par défaut et vous n’utilisez que le lint ou Musea.

<span id="lint"></span>

### Lint

Lorsque `lint` est activé, le module écrit `.oxlint.vize.json` à la racine du projet. C’est l’équivalent Oxlint de la configuration générée par `@nuxt/eslint` : des blocs de règles adaptés à votre version de Nuxt et à vos répertoires. Ajoutez ce fichier à `.gitignore`.

Si le projet n’a encore ni configuration Oxlint ni configuration Vite, le module crée aussi un `oxlint.config.mts` qui charge le fichier généré. Une configuration existante n’est jamais réécrite : chargez-y `.oxlint.vize.json` vous-même.

| Option       | Défaut                | Rôle                                                                                                                        |
| ------------ | --------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `configFile` | `".oxlint.vize.json"` | Emplacement de la configuration générée. Il doit se trouver dans la racine du lint ou dans l’un de ses répertoires parents. |
| `autoInit`   | `true`                | Crée `oxlint.config.mts` lorsqu’aucune configuration Oxlint ou Vite n’existe.                                               |
| `rootDir`    | la racine Nuxt        | Répertoire par rapport auquel les globs générés sont exprimés.                                                              |
| `devtools`   | `{ enabled: "lazy" }` | Affiche le plan de lint résolu dans un onglet Nuxt DevTools.                                                                |

`lint` accepte aussi les options de fonctionnalités de `@nuxt/eslint-config` : `standalone` (`true` par défaut), `import` (`true` par défaut), `typescript`, `stylistic`, `tooling`, `formatters` et `nuxt`.

Indiquez `lint.autoInit: false` lorsque votre configuration de lint se trouve hors de la racine Nuxt. Le [guide du plugin Oxlint](../guide/oxlint.md) explique comment exécuter les règles.

<span id="lint-while-developing"></span>

### Lint pendant le développement

`checker` exécute Oxlint et les règles de lint de Vize à côté du serveur de développement, comme le checker de `@nuxt/eslint` :

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    checker: true,
  },
});
```

| Option        | Défaut                                         | Rôle                                                          |
| ------------- | ---------------------------------------------- | ------------------------------------------------------------- |
| `lintOnStart` | `true`                                         | Analyse tout le projet au démarrage du serveur.               |
| `cache`       | `true`                                         | Après le démarrage, n’analyse que les fichiers modifiés.      |
| `include`     | `<srcDir>/**/*.{js,jsx,ts,tsx,vue}`            | Fichiers examinés.                                            |
| `exclude`     | `**/node_modules/**` et le répertoire de build | Fichiers et répertoires ignorés.                              |
| `emitWarning` | `true`                                         | Affiche les avertissements dans le terminal et le navigateur. |
| `emitError`   | `true`                                         | Affiche les erreurs dans le terminal et le navigateur.        |
| `fix`         | `false`                                        | Applique les corrections sûres avant le rapport.              |
| `formatter`   | `"stylish"`                                    | Format de sortie d’Oxlint.                                    |

<span id="musea"></span>

## Musea

Activez la galerie de composants avec `musea` :

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
    nuxtMusea: {
      route: { path: "/" }, // Musea UI route within __musea__
    },
  },
});
```

En développement, la galerie est servie à `/__musea__/`. Le [guide Musea](../guide/musea.md) couvre les fichiers art, les design tokens et les tests de régression visuelle.

### Où placer les fichiers art

Nuxt parcourt tous les fichiers `.vue` de ses répertoires de composants, et les fichiers art de Musea se terminent aussi par `.vue`. Placez les fichiers `*.art.vue` hors de ces répertoires et indiquez leur emplacement à Musea :

```txt
app/components/Tag.vue
stories/shared/Tag.art.vue
```

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    musea: {
      include: ["stories/**/*.art.vue"],
    },
  },
});
```

Lorsque Musea est activé, le module exclut aussi `**/*.art.vue` de l’analyse des composants de Nuxt : les fichiers art restés à côté des composants ne sont pas enregistrés comme composants.

### Rendre les fonctionnalités Nuxt disponibles dans les aperçus

Les composants dépendent souvent de `NuxtLink`, `useRoute`, `useNuxtApp`, `useRuntimeConfig`, de composables de données ou de composants intégrés à Nuxt. Dans une configuration Vite autonome pour Musea, ajoutez `@vizejs/musea-nuxt` et installez ses simulations depuis `previewSetup` :

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";
import { nuxtMusea } from "@vizejs/musea-nuxt";

export default defineConfig({
  plugins: [
    nuxtMusea({
      route: { path: "/preview" },
      runtimeConfig: { public: { apiBase: "/api" } },
      fetchMocks: {
        "/api/user": { id: 1, name: "Ada" },
      },
    }),
    musea({
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

```ts
// musea.preview.ts
import { installNuxtMuseaMocks } from "@vizejs/musea-nuxt";
import { createI18n } from "vue-i18n";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { apiBase: "/api" } },
  });

  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {/* ... */},
      en: {/* ... */},
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

Dans une application Nuxt, le module n’installe pas ces simulations globalement : elles remplaceraient les `#imports` de Nuxt. Les vrais composables Nuxt sont alors utilisés.

<span id="nuxt-2-and-vue-2"></span>

## Nuxt 2 et Vue 2

Le module prend en charge Nuxt 2, 3 et 4. Le compilateur de Vize produit du code Vue 3 : avec Nuxt 2, le module laisse la compilation des fichiers `.vue` au compilateur existant (`@vitejs/plugin-vue2`, `vue-loader` ou celui de Nuxt) et fournit toujours le lint et Musea. Il en va de même pour les projets Vue plus anciens avec `vueVersion: 0.11`, `vueVersion: 1` ou `vueVersion: "legacy"`.

Les versions sont détectées automatiquement. Si une surcouche les cache à Nuxt Kit, indiquez-les vous-même :

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      nuxtVersion: 2,
      vueVersion: 2,
    },
  },
});
```

## Utiliser directement le plugin Vite

Nuxt construit avec Vite, donc le plugin Vite fonctionne aussi sans le module. Vous perdez alors les passerelles propres à Nuxt et la génération de configuration de lint décrites plus haut :

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Fonctionnement

1. **Plugin Vite :** le module enregistre `@vizejs/vite-plugin`, qui compile les fichiers `.vue`.
2. **Compatibilité avec le plugin Vue :** le plugin expose la même API que `@vitejs/plugin-vue`, si bien que les vérifications internes de Nuxt passent toujours.
3. **SSR :** Vize compile aussi le build serveur et garde séparées les variables d’environnement du client et du serveur.
4. **Transformations Nuxt :** les imports automatiques, les composables et les middlewares passent par la couche de transformation de Nuxt, exécutée après la compilation du composant par Vize.

Les versions publiées de `@vizejs/nuxt` fixent `vize`, `@vizejs/vite-plugin`, `@vizejs/native` et les autres paquets Vize à la même version.

## Exemple : Vue Fes Japan 2026

Le site de [Vue Fes Japan 2026](https://vuefes.jp/2026) utilise le module avec Nuxt 4 :

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: false, // compiler disabled (using Nuxt's default)
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
  },
});
```

Il utilise Musea pour développer et documenter les composants et garde le compilateur par défaut de Nuxt pour le build de production.

## Dépannage

- **Migration d’une configuration dédiée :** les fichiers `vize.config.*` ne sont plus pris en charge. Placez les réglages natifs dans `vue:` de la configuration Vite et conservez les options TypeScript dans `tsconfig.json` ; voir [Configuration](../guide/configuration.md). La clé `vize` du module dans `nuxt.config.ts` reste propre à l’intégration Nuxt.
- **Un `lint.configFile` personnalisé est ignoré :** le fichier doit se trouver dans `lint.rootDir` ou dans l’un de ses répertoires parents. Une configuration placée dans `.nuxt/` ne peut pas appliquer les exclusions du projet d’Oxlint.
- **Liens de ressources dans le HTML SSR de développement :** `dev.stylesheetLinks` conserve les liens de ressources Nuxt valides encodés en URL, comme `%40fs/` et les chemins `assets/` encodés, et supprime les liens qui contiennent des octets nuls ou une traversée de chemin une fois décodés.

Pour tout autre problème, consultez le [dépannage](../guide/troubleshooting.md) ou [signalez un problème](../guide/testing.md).
