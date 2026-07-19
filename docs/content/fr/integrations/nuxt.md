---
title: Nuxt
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Intégration Nuxt

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Testez soigneusement avant d’adopter dans des projets Nuxt.

Vize offre une intégration Nuxt de premier ordre via le module `@vizejs/nuxt` . Cela remplace le compilateur Vue par défaut de Nuxt par le compilateur Rust-native de Vize, offrant les mêmes améliorations de vitesse que les projets Nuxt.

## Commencer

### 1. Installer le module

Installez- `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez le module :

```bash
vp install @vizejs/nuxt
```

Si vous voulez utiliser `pkl` configuration avec pnpm, vous devrez peut-être installer le paquet `vize` lui-même.
`@vizejs/nuxt` installe `vize` qui `vize.pkl` avec la configuration par défaut, mais l’emplacement de `vize.pkl` peut différer lors de l’utilisation de pnpm.

```bash
vp install vize
```

### 2. Enregistrer le module Nuxt

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

### 3. Démarrer Nuxt

Démarrez le serveur de développement comme d’habitude :

```bash
vp run dev
```

Le module injecte `@vizejs/vite-plugin` dans la configuration Vite de Nuxt et conserve les transformations spécifiques à Nuxt
dans le pipeline, de sorte que les importations automatiques, les composants, le middleware et le comportement SSR continuent de fonctionner via
Nuxt.
Pendant le développement, le nettoyage de réponse serveur préserve des liens Nuxt valides encodés en URL, tels que
`%40fs/` et encodés `assets/` chemins, tout en supprimant des chemins décodés à octets nuls ou traversés.

## Options de modules

`@vizejs/nuxt` conserve le simple commutateur `compiler: true | false`, mais les options du module exposent aussi
le compilateur Vize et les ponts de compatibilité Nuxt pour les projets nécessitant un contrôle plus strict :

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

| Option                | Type                                 | Par défaut                 | Description                                                                                                                                                                                                                                                |
| --------------------- | ------------------------------------ | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compatibility`       | `VizeNuxtCompatibilityOptions`       | Détection automatique      | Les overrides détectaient les versions majeures de Nuxt/Vue pour des emballages inhabituels. Nuxt 2 applique par défaut la compatibilité hôte-compilateur Vue 2 ; Nuxt 3/4 est par défaut sur Vue 3. Vue 0.11/1/2 utilisent tous le mode hôte-compilateur. |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions` | `true`                     | Active Vize comme compilateur SFC Vue. Le passage d’un objet redirige les options vers `@vizejs/vite-plugin` tout en conservant les valeurs par défaut Nuxt pour `root`, `devUrlBase`, `scanPatterns`à la demande et la gestion SFC de dépendances.        |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`   | `true`                     | Contrôle le pont de transformation Nuxt pour les importations automatiques, importations de composants, aides i18n et clés async-data stables sur des modules virtuels Vize.                                                                               |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`   | `true`                     | Contrôle le pont UnoCSS pour les modules virtuels Vize. `originalSource: false` désactive la lecture des SFC sources ; `maxBytes` limite l’utilisation de la mémoire.                                                                                      |
| `dev.stylesheetLinks` | `boolean`                            | `true`                     | Permet le nettoyage SSR SSR HTML des liens des liens pour les URL Nuxt générées par Vize uniquement pour les développeurs.                                                                                                                                 |
| `musea`               | `boolean \| MuseaOptions`            | `false`                    | Choisit l’intégration de la galerie Musea. Utilisez `true` pour les paramètres par défaut de Musea ou passez un objet pour configurer des patterns incluants, des jetons, un CSS d’aperçu et le routage.                                                   |
| `nuxtMusea`           | `NuxtMuseaOptions`                   | `{ route: { path: "/" } }` | Documente la forme fictive Nuxt utilisée par les assistants de prévisualisation de Musea. Le module Nuxt n’installe pas globalement la couche mock car cela ombrerait la propre `#imports`de Nuxt.                                                         |

## Configuration avancée

### Nuxt 2 et Legacy Vue

Les projets Nuxt 2 utilisent la sortie du compilateur Vue 2. Le compilateur SFC natif de Vize vise Vue 3, donc le module Nuxt
évite automatiquement de remplacer le compilateur hôte lorsqu’il détecte Nuxt 2. Pour Nuxt 2 Bridge
ou autres configurations Vue 2 basées sur Vite, le plugin Vite reçoit `vueVersion: 2`, qui garde
`@vitejs/plugin-vue2`, `vue-loader`, ou le propre compilateur de Nuxt en charge des fichiers `.vue`.

Le même mode hôte-compilateur est disponible pour les anciens projets Vue via `vueVersion: 0.11`,
`vueVersion: 1`ou `vueVersion: "legacy"`.

Si votre projet enveloppe Nuxt de manière à masquer la version de Nuxt Kit, définissez explicitement la compatibilité
la remise en avant :

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

### Utilisation directe du plugin Vite

Sinon, vous pouvez utiliser directement le plugin Vite. Puisque Nuxt utilise Vite en coulisses, cela fonctionne mais manque de certaines optimisations spécifiques à Nuxt :

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Intégration de la Musea

Le module Nuxt prend également en charge l’intégration de Musea (galerie de composants) :

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

Une fois configurée, la galerie Musea est disponible à `/__musea__/` pendant le développement.

### Placement des fichiers artistiques

L’auto-découverte des composants Nuxt scanne `.vue` fichiers à l’intérieur des répertoires de composants configurés. Parce que
fichiers d’art de Musea se terminent aussi par `.vue`, gardez `*.art.vue` fichiers en dehors de ces répertoires dans les projets Nuxt
et dirigez Musea à cet endroit :

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

Lorsque Musea est activé via `@vizejs/nuxt`, le module exclut également `**/*.art.vue` du scanner de composants
de Nuxt, de sorte que les fichiers hérités colocalisés n’atteignent pas le webpack ou le pipeline de composants Vite de Nuxt.

### Configuration de la prévisualisation pour Nuxt

Les projets Nuxt utilisent souvent des fonctionnalités qui doivent être disponibles dans l’environnement de prévisualisation Musea
(`NuxtLink`, `useRoute`, `useNuxtApp`, `useRuntimeConfig`, composables de données et composants intégrés de Nuxt
). Utilisez `@vizejs/musea-nuxt` dans la configuration autonome de Musea Vite et installez sa couche d’aperçu
mock depuis `previewSetup`:

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
      ja: {
        /* ... */
      },
      en: {
        /* ... */
      },
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

## Comment ça fonctionne

Lorsque le module Nuxt est installé :

1. **Injection de plugins Vite** — Le module `@vizejs/vite-plugin` enregistre comme un plugin Vite, interceptant `.vue` compilation de fichiers.
2. **Calle de compatibilité** — Le plugin expose une API de compatibilité `@vitejs/plugin-vue` , donc les vérifications internes de Nuxt (qui sondent le plugin Vue) fonctionnent correctement.
3. **Prise en charge SSR** — Le `vize_atelier_ssr` de Vize gère la compilation côté serveur. Le plugin isole les variables de l’environnement client et serveur afin d’éviter la contamination croisée.
4. **Fonctionnalités Nuxt préservées** — Les importations automatiques, composables, middlewares et autres fonctionnalités Nuxt fonctionnent via la propre couche de transformation de Nuxt, qui s’exécute après la compilation de Vize.

## Exemple concret

Le site web de la conférence [Vue Fes Japan 2026](https://vuefes.jp/2026) utilise Vize avec Nuxt 4 :

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

Cette configuration utilise Musea pour le développement et la documentation des composants tout en conservant le compilateur par défaut de Nuxt pour les compilations de production.

## Notes

- Vize est en développement actif — tester minutieusement avant d’être utilisé en production sur des projets Nuxt
- La compilation SSR est prise en charge via `vize_atelier_ssr`
- Les fonctionnalités spécifiques à Nuxt (auto-importations, composables, middleware) fonctionnent via la propre couche de transformation de Nuxt
- Le module Nuxt prend en charge, Nuxt 2, Nuxt 3 et Nuxt 4. Nuxt 2 utilise le mode de compatibilité hôte-compilateur car le compilateur SFC natif de Vize cible la sortie Vue 3.
