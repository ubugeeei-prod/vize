---
title: Référence de configuration partagée
---

<!-- Reviewed translation; source: guide/configuration-reference.md -->

<span id="standalone-configuration-reference"></span>

# Référence de configuration partagée

Utilisez `vue:` dans votre configuration Vite existante et `vueCompilerOptions` dans le projet TypeScript. Le [guide de configuration](./configuration.md) présente une mise en place minimale.

## Fichiers de configuration

La CLI et l’éditeur découvrent `vite.config.ts`, `.mts`, `.cts`, `.js`, `.mjs` et `.cjs` dans le projet le plus proche. `package.json`, `tsconfig.json` et `jsconfig.json` délimitent les projets. `--config` sélectionne une configuration Vite.

Les fichiers dédiés `vize.config.*` ne sont plus pris en charge. Un fichier dédié résiduel placé à côté d’une configuration Vite est une erreur, la même erreur de migration qu’un projet contenant uniquement une configuration dédiée. Déplacez ses réglages natifs vers `vue:` et ses options TypeScript vers `tsconfig.json` avant de le supprimer.

Les chemins relatifs appartenant à Vite partent du `root` sélectionné, lui-même résolu depuis le répertoire de configuration. Les entrées CLI explicites et `--tsconfig` partent du répertoire d’exécution. Les options directes du plugin et les fonctions explicitement choisies dans l’éditeur ont priorité sur les réglages partagés.

## Configuration TypeScript

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    templateSyntax: "standard",
  },
  linter: { preset: "happy-path" },
  formatter: { printWidth: 100 },
  languageServer: { hover: true },
});
```

La section `vue` accepte `version`, `compiler`, `typeChecker`, `linter`, `formatter`, `languageServer`, `globalTypes`, `files`, `ignores`, `basePath`, `lib`, `vite`, `musea`, `experimentals`, `dialect`, `name` et `entries`. Utilisez ces noms natifs ; les membres inconnus sont des erreurs. Consultez la [référence du compilateur](./compiler-configuration-reference.md) et les [options des règles](/rules/options.md).

## Résolution des types Vue

`vize check` et le serveur de langage résolvent les types Vue depuis le projet analysé. Déclarez-y la version de Vue souhaitée et gardez `vue`, `@vue/compiler-sfc` et les intégrations comme Nuxt alignés. Les dépendances de compilation de Vize ne choisissent pas la version Vue du projet consommateur. Testez chaque plage Vue prise en charge dans sa propre matrice de paquets.

Conservez les déclarations ambiantes, les fichiers d’auto-imports générés, les alias de chemins et les déclarations Vue `ComponentCustomProperties` dans le `tsconfig.json` du projet. Les `extends` TypeScript standards, y compris les formes par paquet ou tableau, et les `references` conservent leur sens.

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true,
    "checkUnknownComponents": true
  },
  "include": ["src/**/*"]
}
```

Les options Vue prises en charge comprennent `target`, `strictTemplates`, `checkUnknownProps`, `checkUnknownComponents`, `checkUnknownEvents`, `checkUnknownDirectives`, `strictVModel`, `strictCssModules`, `jsxSlots`, `inferComponentDollarEl`, `inferComponentDollarRefs`, `inferTemplateDollarEl`, `inferTemplateDollarRefs`, `inferTemplateDollarSlots`, `inferTemplateDollarAttrs`, `fallthroughAttributes`, `checkRequiredFallthroughAttributes`, `resolveStyleImports`, `resolveStyleClassNames` et `vapor`. Cela ne signifie pas que toutes les options de `vue-tsc` sont prises en charge.

`target` accepte Vue 2.x jusqu’à 2.7, Vue 3.x, `99` et `"auto"`. La sélection automatique suit le paquet Vue installé dans le projet ; `vue.version` explicite dans Vite a priorité. Une valeur non prise en charge produit une erreur de configuration. La rigueur TypeScript reste dans `compilerOptions` ; les options explicites du vérificateur Vize remplacent les valeurs correspondantes du projet.

### Projet TypeScript et limites de prise en charge

Ces réglages proviennent du projet TypeScript propriétaire du fichier source, avec les valeurs JSONC héritées, les `extends` standards par paquet ou tableau et les `references`. `target: "auto"` résout le paquet Vue installé dans ce projet ; une version absente ou non prise en charge produit une erreur de configuration. Une cible numérique Vue 2 inférieure à 2.7 sélectionne Vue 2, exactement 2.7 sélectionne Vue 2.7, et Vue 3.x ou `99` sélectionne la famille Vue 3 moderne. `vue.version` explicitement défini dans Vite, y compris `"3"`, a priorité.

`strictComponentAttrs` est également pris en charge, mais doit être explicitement activé : `strictTemplates` ne l’active pas automatiquement. `plugins` reconnaît uniquement l’identifiant `vue-router/volar/sfc-typed-router` ; il ne charge pas des plugins de langage Vue arbitraires.

`optionsWrapper` n’est pas pris en charge. Dans les outils Vue, il décrit un préfixe et un suffixe de code, pas un interrupteur Options API. La vérification Options API existante de Vize se règle séparément avec `vue.typeChecker.optionsApi`. Cette liste est une correspondance limitée, pas une garantie de parité avec toutes les options `vue-tsc`.

## Entrées expérimentales par périmètre

Utilisez le projet TypeScript du paquet ciblé pour vérifier un monorepo. Les réglages de lint Vue par périmètre peuvent utiliser `vue.entries` :

```ts
export default defineConfig({
  root: "apps/web",
}).withVue({
  entries: [
    {
      files: ["src/trusted/**/*.vue"],
      linter: { rules: { "vue/no-v-html": "off" } },
    },
  ],
});
```

Les entrées acceptent `basePath`, `files`, `ignores` et `linter` ; les autres sections restent communes au projet. Gardez les références de projets TypeScript dans `tsconfig.json`.

## Partager des bases de configuration

Il n’existe pas de `extends` propre à Vize. Réutilisez des objets JavaScript ordinaires :

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./build-options.ts";
import vueConfig from "./vue-options.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

Les `extends` TypeScript standards et les fonctions de configuration Vite restent utilisables.

## Options d’analyse statique

Placez les options natives de lint Vue dans `vue.linter` et les options du serveur de langage dans `vue.languageServer`. Les règles Vue utilisent `vue.linter.rules` ; les règles Oxlint gardent leur emplacement `lint.rules`. Voir [l’exécution du lint](/guide/lint-execution.md) pour les analyses inter-fichiers et sensibles aux types, ainsi que les [options des règles](/rules/options.md). Les champs inconnus des options sont refusés.

```ts
export default defineConfig({}).withVue({
  linter: {
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
  typeChecker: {
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    optionsApi: true,
  },
});
```

`vue.typeChecker.optionsApi` résout les liaisons des templates Vue 3 Options API (`data`, `computed`, `methods`, `inject`, `setup` et `props` d’un `<script> export default { ... }`). Cette fonction est activée par défaut, comme dans `vue-tsc`, et ne s’applique qu’aux composants sans `<script setup>`. Définissez `optionsApi: false` pour la désactiver.

`vue.typeChecker.tsconfig` sélectionne un autre projet TypeScript. Utilisez `vue.typeChecker.corsaPath` uniquement pour sélectionner un exécutable de vérification installé, jamais pour changer les types Vue. `corsaPath` est partagé par `vize check`, le lint sensible aux types et `vize lsp`. Laissez-le absent sauf si vous devez sélectionner un exécutable `lib/tsc` précis.

`vue.typeChecker.servers` limite les processus de vérification parallèles (`1` à `32`). Sans valeur explicite, Vize adapte la concurrence au projet et aux processeurs disponibles.

## Options Musea

Placez la découverte, les routes et les réglages VRT partagés dans `vue.musea`. Les options directes de `musea()` remplacent les valeurs partagées correspondantes.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

Les options de présentation `previewCss`, `previewSetup`, `tokensPath`, `theme` et `storybookOutDir` se placent directement dans `musea()`. `musea-vrt` lit les options partagées `include`, `exclude` et `vrt` dans la même configuration Vite. `vize musea new` crée un exemple sans générer de configuration dédiée.

## Recettes de migration

Les configurations dédiées JSON/PKL et le `extends` propre à Vize sont supprimés. Réutilisez des objets JavaScript importés. Ces recettes conservent les réglages pris en charge pendant la migration.

### Configuration Vite dynamique

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  vue: {
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
    languageServer: {
      lint: true,
      typecheck: false,
      editor: false,
      formatting: false,
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
    },
  },
}));
```

### Migrer les anciens réglages JSON et PKL

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: { scanPatterns: ["src/**/*.vue"] },
  linter: { preset: "happy-path" },
  typeChecker: { enabled: true, strict: true },
  languageServer: { lint: true, typecheck: false, editor: false, formatting: false },
  musea: { include: ["src/**/*.art.vue"], basePath: "/__musea__" },
});
```

### Projets de paquets distincts

`entries` ne définit que des périmètres de lint. Placez les réglages du vérificateur et du formateur dans les projets Vite et TypeScript de chaque paquet. Les deux fichiers ci-dessous appartiennent à des projets distincts.

```ts
// apps/web/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  name: "web app",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100 },
  typeChecker: { tsconfig: "tsconfig.app.json" },
});
```

```ts
// packages/ui/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({
  name: "ui package",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100, singleQuote: true },
});
```

### Analyse statique et processus de vérification

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
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

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  typeChecker: { servers: 1 },
});
```

### Réglages des workflows existants

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```
