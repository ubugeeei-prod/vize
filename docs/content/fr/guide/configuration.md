---
title: Configuration
description: Partagez les réglages Vue depuis votre configuration Vite et votre projet TypeScript.
---

<!-- Reviewed translation; source: guide/configuration.md -->

# Configuration

Placez les réglages Vue sous `vue:` dans `vite.config.*` et les réglages TypeScript dans `tsconfig.json`. La CLI, l’intégration Vite et l’extension de l’éditeur lisent ces fichiers de projet.

## Conserver votre configuration Vite+

Enveloppez votre configuration existante avec l’assistant Vize :

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
};

const vueConfig = {
  compiler: { sourceMap: true },
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

`withVue` conserve les options, les plugins et les hooks Vite+ existants. Les objets, fonctions, promesses et fonctions asynchrones sont pris en charge. Une section `vue: { ... }` en ligne est équivalente. Le [guide de configuration Vue partagée](/guide/vue-configuration.md) décrit les chemins, les périmètres et la composition.

| Emplacement | Fonction | Commande |
| --- | --- | --- |
| `vue.compiler` | Compilation Vue | `vp dev`, `vp build` |
| `vue.linter` | Règles lint Vue | `vp run lint` |
| `vue.formatter` | Formatage Vue | `vp run fmt:check` |
| `vue.typeChecker` | Réglages explicites du vérificateur Vue | `vp run typecheck` |

Utilisez `vp run check` pour les tâches Vize combinées. Les commandes intégrées `vp check`, `vp lint` et `vp fmt` conservent le fonctionnement propre à Vite+. Un script existant peut renommer une tâche générée en `vize:<nom>` ; consultez les [noms des tâches](/guide/vite-plus.md#tasks).

## TypeScript et vérification des templates Vue

Conservez votre projet TypeScript, y compris ses `extends` et `references` standards. Placez les options propres aux templates dans `vueCompilerOptions` :

```json
{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "jsx": "preserve"
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}
```

Les projets utilisant `vue-tsc` peuvent garder leur organisation TypeScript. `compilerOptions` définit la rigueur TypeScript ; `vueCompilerOptions` contient les options Vue. La [référence](./configuration-reference.md) présente les options prises en charge et les réglages explicites de Vize.

## Modifier une règle

Placez les règles Vue dans `vue.linter.rules` et celles d’Oxlint dans `lint.rules`. Les [options des règles](/rules/options.md) décrivent les paramètres et le [catalogue](../rules/all.md) donne les exemples complets.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { rules: { "no-debugger": "error" } },
}).withVue({
  linter: { rules: { "vue/no-v-html": "error" } },
});
```

<span id="lint-rule-options"></span>

Les champs inconnus des options de règles sont refusés.

## Choisir les fonctions à adopter

Placez les réglages natifs sous `vue.compiler`, `vue.typeChecker`, `vue.linter` et `vue.formatter`. Vize formate Vue et Oxfmt les autres fichiers. Consultez la [répartition et la gestion des conflits](/guide/vite-plus.md#lint-and-formatter-ownership).

## Vite classique

Conservez la configuration Vite existante et remplacez le plugin de compilation Vue :

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vue: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
  },
});
```

<span id="standalone-cli"></span>

## CLI et éditeur

Lancez les commandes depuis la racine du paquet ciblé. L’éditeur résout le projet de chaque document dans l’espace de travail sélectionné et lit ses réglages Vite et TypeScript. Chaque paquet d’un monorepo conserve son `tsconfig.json`, ses références et sa configuration Vite. `--tsconfig` sélectionne explicitement un autre projet TypeScript.

```bash
vp exec vize check
vp exec vize lint src
vp exec vize fmt --check src
```

Les chemins relatifs natifs dans la configuration Vite partent du `root` Vite sélectionné. Un `root` relatif part du répertoire de configuration. Les fichiers et `--tsconfig` explicitement passés à la CLI restent relatifs au répertoire d’exécution. Les options CLI et les fonctions explicitement choisies dans l’éditeur ont priorité sur les réglages partagés.

Les exclusions globales partagées retirent les fichiers de la recherche CLI et du lint de l’éditeur. Un fichier exclu ouvert conserve les diagnostics de syntaxe, de types et de navigation. L’ordre et le sens des motifs, y compris la négation `!`, sont préservés.

## Migrer les anciennes configurations

Les fichiers `vize.config.*` et la section racine `vize:` ne sont plus pris en charge. Déplacez les réglages natifs sous `vue:` dans `vite.config.*`, et les options du projet TypeScript dans `tsconfig.json`. Après vérification, supprimez les fichiers dédiés. Un ancien fichier dédié placé à côté d’une configuration Vite est ignoré ; un projet qui ne contient qu’un fichier dédié reçoit une erreur de migration. `vize init` et `vize setup` préservent les fichiers originaux et expliquent la migration.

Il n’existe pas de `extends` propre à Vize. Importez des objets JavaScript ordinaires et transmettez-les à `defineConfig(vpConfig).withVue(vueConfig)` pour partager vos réglages. Les `extends` et `references` TypeScript standards restent valides.

<span id="fichiers-de-configuration"></span>
<span id="configuration-typescript"></span>
<span id="résolution-de-type-vue"></span>
<span id="entrées-expérimentales-sur-le-plat"></span>
<span id="configuration-pkl"></span>
<span id="configuration-json"></span>
<span id="options-du-compilateur"></span>
<span id="syntaxe-des-modèles"></span>
<span id="mode-de-sortie-jsx-tsx"></span>
<span id="directives-par-composant"></span>
<span id="préséance"></span>
<span id="diagnostic"></span>
<span id="dialecte-vue"></span>
<span id="options-d-analyse-statique"></span>
<span id="musea-options"></span>

## Référence détaillée

La [référence partagée](./configuration-reference.md) décrit la recherche des fichiers, les périmètres, la résolution des types Vue et les réglages LSP/Musea. Consultez la [référence du compilateur](./compiler-configuration-reference.md) et les [fonctions expérimentales](/guide/experimentals.md) pour leurs options.

> [!NOTE]
> La CLI et l’éditeur lisent les projets Vite et TypeScript existants. Migrez les anciens réglages dédiés avant de supprimer leurs fichiers.
