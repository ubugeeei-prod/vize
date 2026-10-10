---
title: Bien démarrer
description: Ajoutez Vize à Vue avec votre configuration Vite et votre projet TypeScript.
---

<!-- Reviewed translation; source: getting-started.md -->

# Bien démarrer

Ajoutez Vize à votre application Vue 3 avec [Vite+](https://viteplus.dev/guide/install). Gardez les réglages des outils dans `vite.config.ts` et le projet TypeScript dans `tsconfig.json`. Vize est en développement ; vérifiez son [niveau de support](./stability.md) avant de l'adopter.

## 1. Installer l'intégration

Dans un projet Vue existant équipé de Vite+ :

```bash
vp install -D @vizejs/vite-plugin
```

L'intégration utilise la version Vite+ du projet (0.2.3 ou ultérieure). Pour Vite classique, consultez la [migration du plugin](/guide/migration.md#vite-plugin) ; pour Nuxt, consultez l'[intégration Nuxt](./integrations/nuxt.md).

## 2. Modifier `vite.config.ts`

La fonction ajoute le compilateur Vize et les tâches natives de vérification. Retirez l'import de l'ancien plugin Vue et `vue()` de `plugins`, puis conservez les alias, le serveur, les tests et les autres plugins. Le [guide de migration](/guide/migration.md) montre les exemples avant et après.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

## 3. Construire et vérifier l'application

```bash
vp dev
vp build
vp run check
```

`vp run check` combine la vérification des types Vue, le lint et le formatage avec Oxlint et Oxfmt. Utilisez les tâches ciblées pendant le développement :

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

Appliquez les corrections avec `vp run check -- --fix`. Utilisez `vp run` pour les tâches Vize générées : `vp check`, `vp lint` et `vp fmt` exécutent les outils propres à Vite+. Vize conserve les scripts existants et génère `vize:<nom>` en cas de conflit ; consultez les [noms et remplacements](/guide/vite-plus.md#tasks).

## Choisir la suite

- [Configurer une règle ou une option du compilateur](./guide/configuration.md).
- [Migrer les outils existants](/guide/migration.md) avec des exemples avant/après complets.
- [Comprendre un diagnostic lint](./rules/all.md) avec des exemples Vue incorrects et corrigés.
- [Explorer les composants](./guide/ui/index.md), leurs recettes et leur API.
- [Prévisualiser les composants avec Musea](./guide/musea.md).
- [Configurer l'éditeur](/guide/vite-plus-editor.md) avec les réglages natifs partagés.

Les guides qui ne sont pas encore traduits s'ouvrent en anglais.

> [!NOTE]
> La découverte de la configuration Vite par la CLI et l'éditeur natifs, ainsi qu'init sans fichier dédié, sont en préparation pour la prochaine version. D'ici là, les outils natifs publiés utilisent encore le format dédié existant pour les réglages partagés personnalisés. La [référence](./guide/configuration-reference.md) décrit ce format.

## Utiliser Vize sans Vite+

La [CLI autonome](./guide/cli.md) fournit `vize lint`, `vize fmt` et `vize check`. Lancez-les depuis la racine du paquet ciblé et gardez le projet TypeScript dans `tsconfig.json`. Les commandes natives peuvent partager l'objet `vize` à la racine de `vite.config.*` ; consultez la [configuration CLI](./guide/configuration.md#standalone-cli) pour la recherche et ses limites actuelles.

Prévisualisez l'installation interactive avec `vpx vize init --dry-run`, puis lancez `vpx vize init` pour choisir les fonctions. Le [guide d'installation](/guide/init.md) décrit la détection et les limites d'édition. Init utilise les réglages du projet et les valeurs par défaut, sans créer de configuration Vize dédiée.

<span id="configurer-un-projet-existant"></span>
<span id="choisir-une-configuration-manuelle"></span>
<span id="consulter-les-guides-spécialisés"></span>
