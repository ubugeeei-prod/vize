---
title: Void
description: Utilisez Vize dans une application Void avec les pages @void/vue, le SSR sur workerd et les types générés par Void.
---

<!-- Generated translation; source: integrations/void.md -->

# Void

[Void](https://void.cloud) construit des applications full-stack sur Vite et les exécute sur workerd de Cloudflare. Son adaptateur Vue, `@void/vue`, rend `pages/` côté serveur et les hydrate dans le navigateur. Ajoutez `vize()` à côté : Vize compile chaque fichier `.vue`, et Void garde le routage, les loaders, les îlots et les bindings.

> [!WARNING]
> Vize est en développement actif. Comparez la sortie de build et les diagnostics sur votre propre application avant de l'adopter, et consultez l'[état du support](../stability.md).

## Installation

### 1. Installer le plugin

```bash
npm install -D @vizejs/vite-plugin
```

### 2. Ajouter `vize()` à `vite.config.ts`

```ts
import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
```

`voidVue()` embarque son propre `@vitejs/plugin-vue`. Inutile de le retirer : `vize()` le remplace pour qu'un seul compilateur traite chaque SFC. Si vous définissez `exclude`, plugin-vue reste et compile les fichiers que Vize ignore.

### 3. Lancer Void comme d'habitude

```bash
npx vite dev
npx vite build
```

Aucun autre changement n'est nécessaire :

- **Rendu serveur sur workerd.** Vue et `vue/server-renderer` sont intégrés au worker, car workerd n'a pas de `node_modules` à importer.
- **CSS et préchargements des routes.** Le manifeste client est indexé par chemin source (`pages/index.vue`), la clé que Void utilise pour ajouter la feuille de style d'une route au HTML rendu côté serveur.
- **HMR dans `vite dev`.** Modifier un template conserve l'état du composant. Les props des loaders restent en place.
- **Props de page.** `defineProps<Props>()` avec `Props` importé d'un loader `.server.ts` se compile comme avec plugin-vue.

## Vérification des types

Void génère `.void/tsconfig.json` avec les types des routes, de l'environnement et des bindings ; votre `tsconfig.json` l'étend. Exécutez `void prepare` une fois (ou lancez le serveur de développement) pour créer ces fichiers, puis vérifiez :

```bash
npx void prepare
vize check
```

`vize check` lit les `files` et `paths` générés ; dans les fichiers `.vue`, ces éléments sont typés comme avec `vue-tsc` :

- les props des loaders via `InferProps<typeof loader>`,
- `fetch` de `void/client` (chemins, méthodes et corps de réponse des routes),
- `env` de `void/env` et les modules de binding comme `void/kv`.

## Lint et formatage

`vize lint` et `vize fmt` trouvent vos fichiers depuis la racine du projet et ignorent ce que liste `.gitignore`. Le `.gitignore` de Void couvre `dist`, `.void`, `.wrangler` et `.void-wrangler.jsonc` : les fichiers générés ne sont ni analysés ni reformatés.

## Éditeur

Le [serveur de langage](../guide/language-server.md) utilise le même `tsconfig.json`. Exécutez `void prepare` avant d'ouvrir le projet pour que les types générés existent.

## Comment c'est testé

À chaque modification du plugin Vite ou du compilateur natif, la CI construit une application Void 1.1.12 épinglée avec `@void/vue` (`tools/support/compat/void/fixtures/void-vue`) et vérifie :

- le build de production et le SSR dans `vite preview` sur workerd (CSS des routes et binding KV compris) ;
- le SSR en développement, l'hydratation et un HMR qui conserve l'état ;
- que `vize check` et `vue-tsc` signalent les mêmes erreurs ;
- `vize lint` et `vize fmt --check`.

Nuxt sur Void (`voidPlugin()` dans `nuxt.config.ts`) n'est pas encore couvert par cette fixture. Utilisez l'[intégration Nuxt](./nuxt.md) dans ce cas.
