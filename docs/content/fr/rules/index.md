---
title: Règles
description: Activez les règles de lint de Vize avec un préréglage, puis ajustez-les une à une.
---

<!-- Generated translation; source: rules/index.md -->

# Règles

Le linter de Vize vérifie les composants Vue monofichiers : templates, scripts, styles et liens entre fichiers. Choisissez un préréglage, puis renforcez, assouplissez ou désactivez des règles individuelles.

## Activer le lint

Avec Vite+, importez `defineConfig` depuis `@vizejs/vite-plugin/vite-plus`, configurez `vue.linter` et lancez `vp run lint`. Cette commande exécute les règles de Vize et Oxlint ensemble.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
    linter: {
        preset: "happy-path",
        rules: {
            "vue/no-v-html": "off",
            "a11y/alt-text": "error",
        },
    },
});
```

Chaque règle accepte `off`, `warn` ou `error`. Les règles qui ont des réglages les lisent dans `vue.linter.ruleOptions` ; voir [Options des règles](/rules/options.md).

Si le projet a déjà un script nommé `lint`, utilisez `vp run vize:lint`. La commande intégrée `vp lint` est le vérificateur de Vite+ et n’exécute pas les règles de Vize.

## Préréglages

Un préréglage est un ensemble de règles de départ. `happy-path` est celui par défaut.

| Préréglage | Règles | Ce qu’il active |
| --- | ---: | --- |
| `essential` | 85 | Validité des templates et des directives, suppressions de Vue 3 (`vue/no-deprecated-*`, `script/no-deprecated-*`), justesse des macros de script, de `nextTick` et de l’Options API, hooks de cycle de vie, watchers et `expose` enregistrés après `await`, imports depuis `vue` plutôt que `@vue/*`, composants inutilisés, `vue/no-v-html`, `vue/no-unsafe-url`, `html/id-duplication` |
| `happy-path` (par défaut) | 147 | `essential`, plus les conventions de mise en forme des templates (dont la forme courte de `v-bind` et les événements de composant avec trait d’union), l’accessibilité, la conformité HTML, les risques SSR, les propriétés inutilisées, les emits déclarés et les props et emits typés |
| `ecosystem` | 155 | `happy-path`, plus les vérifications des bibliothèques de l’écosystème |
| `nuxt` | 213 | `opinionated`, adapté aux auto-imports de Nuxt |
| `opinionated` | 214 | `happy-path`, plus des conventions plus strictes, des props typées, `defineProps`/`defineEmits` basés sur les types, l’ordre des macros de `<script setup>`, des noms d’événements en kebab-case, des guillemets cohérents pour les clés de props et emits, `useTemplateRef`, `defineSlots`, l’hygiène des valeurs par défaut des props et des emits, `defineOptions` et les règles de style `css/*` |
| `incremental` | 0 | Rien. Partez de là pour n’activer que les règles que vous nommez. |

Les nombres excluent les règles inter-fichiers. Quand `compiler.compatibility.vueVersion` vaut `2` ou `2.7`, les vérifications des suppressions de Vue 3 sont désactivées.

## Trouver une règle

| Page | Contenu |
| --- | --- |
| [Toutes les règles](./all.md) | Toutes les règles dans un index par catégorie, avec le plus petit préréglage qui active chacune |
| [Vue](./vue.md) | Templates, directives, composants et blocs des SFC, et migration vers Vue 3 |
| [Type et script](./type-and-script.md) | `<script setup>`, macros du compilateur, réactivité et vérifications typées |
| [HTML](./html.md) | Validité du HTML et balisage sémantique |
| [Accessibilité](./accessibility.md) | ARIA, interactions au clavier, libellés, repères et alternatives aux médias |
| [SSR](./ssr.md) | Risques du rendu serveur et de l’hydratation |
| [Vapor](./vapor.md) | Contraintes des composants compilés en mode Vapor |
| [Écosystème et Nuxt](./ecosystem.md) | Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils et Void Vue |
| [CSS](./css.md) | Vérifications de justesse et de maintenabilité du CSS standard |
| [Musea](./musea.md) | Stories `.art.vue`, variantes et jetons de design de Musea |
| [petite-vue](./petite-vue.md) | Pages HTML qui utilisent petite-vue |
| [Inter-fichiers](./cross-file.md) | Vérifications entre composants, providers et routeurs, activées avec `crossFile: true` |
| [Options des règles](/rules/options.md) | `ruleOptions` typées et leurs valeurs par défaut |
| [Migrer depuis ESLint](/rules/migration.md) | Noms des règles d’`eslint-plugin-vue` et leurs équivalents Vize, avec les différences |

Chaque page de règle commence par ce que la règle vérifie, sa gravité par défaut et ses préréglages. Elle montre ensuite un mauvais exemple, que la règle signale, et un bon exemple, qu’elle accepte, chacun avec une phrase d’explication, puis la configuration qui active la règle. Les exemples sont ceux que la suite de tests du linter exécute.

## Règles typées

Les règles `type/` lisent les informations de type du projet TypeScript décrit par `tsconfig.json`. En plus d’un préréglage ou d’une entrée `rules` qui les active, elles ont besoin de `typeAware: true` dans `vue.linter`. Déclarez les noms d’environnement partagés dans `compilerOptions.types` ou avec des références de projet, et non dans une liste `globals` séparée dans la configuration de Vize.
