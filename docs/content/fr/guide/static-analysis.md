---
title: Analyse statique
---

<!-- Generated translation; source: guide/static-analysis.md -->

# Analyse statique

La pile d’analyse de Vize est partagée par le compilateur, linter, le vérificateur de types, le serveur d’éditeur et les outils Musea
. L’objectif est de parseser un SFC Vue une fois, de conserver des informations sémantiques riches, puis de les réutiliser
pour le diagnostic et la génération de code au lieu de traiter chaque commande comme un outil séparé.

Les exemples ci-dessous supposent que le paquet `vize` npm est installé et appelé à partir de scripts de projet, ce qui
est le flux de travail recommandé pour les applications.

## Pipeline

| Couche   | Ce que ça fait                                                                                         | Utilisé par                                          |
| -------- | ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------- |
| Armature | Tokenise et analyse les modèles Vue et la structure SFC                                                | compilateur, linter, formateur                       |
| Croquis  | Construit des périmètres, des métadonnées de liaison, des informations macro et des graphiques croisés | Compilation, LINT, vérifications sensibles aux types |
| Patine   | Utilise Vue, script, CSS, a11y, SSR, Vapor, Musea, et des règles de type sensibles aux types           | `vize lint`, diagnostic de l’éditeur, pont Oxlint    |
| Canon    | Génère un TypeScript virtuel et associe les diagnostics aux fichiers Vue                               | `vize check`, vérification du type d’éditeur         |
| Maestro  | Expose les fonctionnalités de diagnostic et d’éditeur via LSP                                          | `vize lsp`, VS Code, Zed                             |

Cela signifie que l’analyse statique n’est pas seulement un linting. Les liaisons de templates, les macros du compilateur, les métadonnées
composants, les relations de fournisseur/injection, le flux de réactivité, le TypeScript virtuel généré et
les métadonnées de la galerie de composants dépendent toutes du même travail d’analyse de bas niveau.

Pour les noms de règles concrets, les paramètres par défaut et les codes de diagnostic croisés pouvant être émis, voir
[Rules](../rules/index.md).

## Linting

Commencez par le préréglage par défaut :

```json
{
  "scripts": {
    "vize:lint": "vize lint src"
  }
}
```

```bash
vp run vize:lint
```

Utilisez `essential` pour l’IC uniquement de la correction, `happy-path` pour le bundle recommandé par défaut,
`opinionated` quand vous voulez des conventions plus fortes, `nuxt` pour les hypothèses conscientes de Nuxt, et
`incremental` quand vous ne voulez que des règles explicitement configurées pour exécuter.

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:fix": "vize lint --fix src",
    "vize:lint:json": "vize lint --format json src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:opinionated
vp run vize:lint:fix
vp run vize:lint:json
```

Optez pour des vérifications croisées et sensibles au type seulement après que le chemin de lint de base est stable :

```json
{
  "scripts": {
    "vize:lint:cross-file": "vize lint --cross-file src",
    "vize:lint:cross-file-tree": "vize lint --cross-file --cross-file-tree src",
    "vize:lint:strict-reactivity": "vize lint --strict-reactivity src"
  }
}
```

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
vp run vize:lint:strict-reactivity
```

Le linting entre fichiers analyse des relations telles que fournir/injecter et le flux de réactivité à travers un ensemble de fichiers Vue
. `--strict-reactivity` active la règle native de perte de réactivité soutenue par les damiers, donc attendez-vous à ce qu’elle soit
plus lente que les règles classiques de template et script lint.

## Superposition de réactivité

Croquis expose une superposition de réactivité stable pour chaque SFC analysé : sources réactives, exigences `.value`
, sites de perte de réactivité et arêtes de graphes d’effets avec mappages de sources. Le même modèle compact
JSON alimente les diagnostics, les rapports, les surfaces de l’éditeur et l’onglet **Réactivité** du Playground.

## Modèle de règle de patine

La patine est la couche de règle de peluche. Les règles sont de petits visiteurs sur la source SFC, la racine du modèle,
les éléments du modèle, les directives, `v-for`, `v-if`et les interpolations. Chaque règle contient les métadonnées
son nom de règle, sa catégorie, sa gravité par défaut, son texte d’aide et sa capacité à être corrigée. Les préréglages ne sont que
registres qui décident ensemble quelles règles sont activées.

| Superficie                  | Exemples de règles                                                                           | Ce qu’ils couvrent                                                  |
| --------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Correction de la vue        | `vue/require-v-for-key`, `vue/valid-v-model`, `vue/no-use-v-if-with-v-for`                   | Sémantique de modèles locales à un composant                        |
| Sécurité Vue                | `vue/no-v-html`, `vue/no-unsafe-url`                                                         | Des ennuis HTML et URL sujets à XSS                                 |
| Structure de la vue         | `vue/sfc-element-order`, `vue/require-scoped-style`, `vue/no-unused-components`              | Forme du SFC, utilisation des composants et maintenabilité          |
| Conventions d’écriture      | `script/no-options-api`, `script/no-get-current-instance`, `script/prefer-import-from-vue`   | API Vue Composition et conventions de macro du compilateur          |
| CSS                         | `css/no-important`, `css/no-hardcoded-values`, `css/prefer-logical-properties`               | Blocs de style et CSS adapté aux systèmes de conception             |
| Accessibilité               | `a11y/img-alt`, `a11y/anchor-has-content`, `a11y/label-has-for`                              | Balisage accessible et motifs d’interaction                         |
| HTML                        | `html/deprecated-element`, `html/id-duplication`, `html/no-empty-palpable-content`           | Validité HTML et balisage sémantique                                |
| SSR                         | `ssr/no-browser-globals-in-ssr`, `ssr/no-hydration-mismatch`                                 | Dangers de rendu serveur/client                                     |
| Vapeur                      | `vapor/no-vue-lifecycle-events`, `vapor/no-inline-template`, `vapor/require-vapor-attribute` | Contraintes de gabarit orientées vapeur                             |
| Musea                       | `musea/require-title`, `musea/valid-variant`, `musea/prefer-design-tokens`                   | Galerie de composants et création de variantes                      |
| Analyse en fonction du type | `type/require-typed-props`, `type/require-typed-emits`, `type/no-reactivity-loss`            | Règles nécessitant un contexte sémantique ou appuyé sur des damiers |

Les préréglages intégrés sont destinés à soutenir l’adoption par étapes :

| Préréglage    | Forme                                                                                   |
| ------------- | --------------------------------------------------------------------------------------- |
| `essential`   | Correction Vue axée sur les erreurs, sécurité et vérifications HTML minimales           |
| `happy-path`  | Bundle par défaut pour la correction, la sécurité, a11y, SSR, vérifications sémantiques |
| `opinionated` | `happy-path` plus des conventions, règles de script et règles de type plus fortes       |
| `nuxt`        | Règles opiniâtes ajustées aux hypothèses d’importation automatique de Nuxt              |
| `incremental` | Point de départ vide pour une adoption guidée par l’hôte, règle par règle               |

## Pragmas de migration et règles de coutume

Patina accepte les pragmas existants de désactivation ESLint pour faire correspondre les noms de règles, y compris
`eslint-disable`, `eslint-enable`, `eslint-disable-next-line`et `eslint-disable-line`. Cela permet
projets de migrer des règles comme `vue/require-v-for-key` sans réécrire chaque commentaire de suppression
au départ.

Les modules de règles JavaScript locaux au projet ne sont pas encore une API d’exécution Vize stable. Pendant la migration, gardez
ces règles dans ESLint ou Oxlint et exécutez-les à côté de `vize lint`, ou utilisez le préréglage `incremental` pour
n’activer que les règles Vize intégrées qui correspondent déjà à votre politique. L’objet de configuration `rules` contrôle
sévérités intégrées des règles Vize par nom.

Dans le cas courant d’interdiction d’un global environnement-exécution (règles ESLint typiques de sidecar telles que
`no-access-process`, `no-access-local-storage`ou `no-restricted-globals` contre `localStorage` /
`sessionStorage`), activez la règle intégrée de `script/no-restricted-globals` opt-in au lieu de garder
ESLint installé uniquement pour celles-ci. Sa liste de refus par défaut est `process`, `localStorage`, et
`sessionStorage`, rapportée sur chaque référence nue.

Deux règles de script acceptent également la configuration locale du projet sous `linter.ruleOptions` (#1891), afin que les équipes puissent
imposer leurs propres conventions architecturales via `vize lint`. `script/no-restricted-globals`
prend une liste `globals` qui **remplace** la liste par défaut intégrée ; `script/no-restricted-members` est
désactivé jusqu’à configuration et signale `<object>.<property>` accès à une liste `members`. Les options sont tapées
(`name` / `object` / `property` plus un `message`optionnel , avec des clés inconnues rejetées) ; Un
`message` manquant revient à un avis générique.

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." }
        ]
      }
    }
  }
}
```

## Règles croisées de fichiers

L’analyse croisée se trouve à Croquis et est exposée à la lintature grâce au diagnostic de la patine. Il est
opt-in car il construit un registre de modules, un graphe d’importation, un graphe d’utilisation des composants, ainsi que des index de
supplémentaires à travers tous les fichiers Vue analysés.

Aujourd’hui, `vize lint --cross-file` permet la correspondance fourni/injectée, des vérifications uniques d’identification des éléments, le suivi de la réactivité
et l’analyse asynchrone des conditions de race. `--cross-file-tree` imprime l’arbre de
fournisseur/injection par-dessus ces diagnostics.

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

Le moteur cross-file de bas niveau est plus large que la surface actuelle de la CLI :

| Option de cross-file      | Diagnostics ou faits prévus                                                                        |
| ------------------------- | -------------------------------------------------------------------------------------------------- |
| `provide_inject`          | Injections non appariées, fournitures inutilisées, avertissements de string-key, flux non réactifs |
| `unique_ids`              | Des identifiants dupliqués et des identifiants non uniques introduits à l’intérieur des boucles    |
| `reactivity_tracking`     | Déstructuration des hélices, aliasing et perte de réactivité croisée                               |
| `race_conditions`         | Mises à jour d’état asynchrones pouvant passer rapidement par l’état fourni ou partagé             |
| `fallthrough_attrs`       | `$attrs`, `inheritAttrs`, et les risques de chute à racines multiples                              |
| `component_emits`         | Émissions non déclarées, émissions non utilisées, et auditeurs sans producteur                     |
| `event_bubbling`          | Des événements qui débordent les frontières des composants sans être gérés                         |
| `server_client_boundary`  | Utilisation de l’API de navigateur et risques d’hydratation autour des frontières SSR/client       |
| `error_suspense_boundary` | Composants asynchrones sans limites de suspense ou d’erreur utiles                                 |
| `circular_dependencies`   | Cycles d’importation et chaînes d’importation profondes                                            |
| `component_resolution`    | Utilisation de composants non enregistrés ou non résolus                                           |
| `props_validation`        | Accessoires manquants et types d’accessoires pour enfants                                          |

L’objectif est de maintenir le linting d’un seul fichier rapide par défaut, d’exposer explicitement les groupes de fichiers croisés au fur et
ils mûrissent, et de router les faits de projet à haute confiance dans le même flux de diagnostic utilisé par la
CLI, le pont Oxlint et le serveur éditeur.

## Vérification de type

`vize check` génère un TypeScript virtuel pour les SFC Vue et demande aux sessions du projet Corsa des diagnostics
. Il vérifie `.vue`, `.ts`, `.tsx`et `.d.ts` entrées et redistribue les diagnostics aux fichiers sources
originaux.

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:src": "vize check src",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:json": "vize check --format json --quiet",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:profile": "vize check --profile src",
    "vize:check:single-server": "vize check --servers 1 src",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:src
vp run vize:check:app
vp run vize:check:json
```

Lorsqu’aucun chemin n’est fourni, `vize check` lit `tsconfig.json` `files`, `include`, et `exclude`
champs si une configuration de projet est disponible. Utilisez `--show-virtual-ts` pour déboger le code généré et
`--profile` lorsque vous avez besoin de timing et d’artefacts de fichiers virtuels sous `node_modules/.vize`.

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

La sortie des déclarations est disponible à partir du projet de vérification matérialisée :

```bash
vp run vize:check:declarations
```

Les valeurs de modèles à l’échelle du projet et les fichiers de déclaration générés doivent être visibles via TypeScript
la configuration du projet. Placez les déclarations ambiantes sous un chemin inclus par votre `tsconfig` et passez
ce fichier projet au vérificateur lorsque nécessaire :

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
    $route: { path: string };
  }
}
```

```bash
vp run vize:check:app
```

## Scripts de paquets npm vs Rust CLI

Le paquet npm `vize` est destiné aux scripts de paquet et utilise la liaison NAPI empaquetée :

```json
{
  "scripts": {
    "vize:lint": "vize lint src",
    "vize:check": "vize check src --strict",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Le CLI Rust dispose actuellement d’une surface de vérification de type plus complète soutenue par le projet :

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

Utilisez des scripts de paquets npm lorsque vous souhaitez des flux de travail installables dans une application. Utilisez la CLI Rust lorsque
vous avez besoin de `check-server`, LSP, gestion de l’IDE ou du chemin de diagnostic de projet soutenu par Corsa à travers
fichiers Vue et TypeScript.

## Oxlint

Utilisez `oxlint-plugin-vize` lorsque votre équipe exécute déjà Oxlint et veut des diagnostics compatibles Vue dans la même commande
:

```bash
vp install -D oxlint oxlint-plugin-vize
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

## Parcours d’adoption

1. Ajoutez un script de paquet `vize:lint:ci` comme `vize lint --preset essential src` à CI.
2. Passez à `happy-path` ou `opinionated` une fois les diagnostics de correction propres.
3. Ajoutez un script de paquet `vize:check` avec votre `tsconfig.json`de projet.
4. Activez d’abord le linting de l’éditeur, puis vérifiez les types une fois que la sortie CI est stable.
5. Ajoutez des vérifications croisées et strictes de réactivité pour les projets bénéficiant d’une analyse plus approfondie.

Pour une seule porte de qualité, un script de paquet `vize:ready` exécutant `vize ready src` exécute `fmt

- -write`, `lint`, `check`et `build` dans l’ordre et s’arrête à la première étape défaillante.
