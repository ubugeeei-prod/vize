---
title: Référence de configuration du compilateur
---

<!-- Reviewed translation; source: guide/compiler-configuration-reference.md; scope: reference relocation -->

# Référence de configuration du compilateur

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


