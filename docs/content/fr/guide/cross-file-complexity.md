---
title: Complexité croisée de fichiers
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# Complexité croisée de fichiers

Le rapport de complexité croisée de fichiers de Vize est un résumé du graphe de projet produit par
Croquis. Ce n’est pas une règle de diagnostic en soi : c’est un score explicable que les outils en
aval peuvent afficher dans des rapports, dans le Playground et dans de futures vérifications à seuil.

Le modèle associe trois signaux de complexité à Vue :

- Nombre de chemins du template : la complexité cyclomatique propre à chaque composant, calculée par
  l’analyse S2 `template-complexity` de Davinci. Elle compte chaque condition `v-if` / `v-else-if`,
  chaque `v-for`, et chaque `&&`, `||`, `??` et `?:` dans les expressions que le template évalue.
- Flux de contrôle imbriqué : la complexité cognitive propre à chaque composant. Les branches et les
  boucles coûtent d’autant plus qu’elles sont imbriquées dans des régions `v-if`, `v-for` ou de slot
  à portée (scoped slot).
- Flux de données aux frontières des composants : les arêtes de props, de provide/inject et réactives
  restent visibles comme signaux transfrontaliers au lieu d’être aplaties dans un seul fichier.

La définition des métriques et les seuils fixés sur le corpus se trouvent dans
[`complexity-metrics.md`](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/plan/complexity-metrics.md).

## Scores

Le rapport expose à la fois les signaux bruts et les scores dérivés.

| Champ             | Signification                                                                                                                           |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Somme de la complexité cyclomatique propre au template de chaque composant.                                                             |
| `cognitiveScore`  | Somme de la complexité cognitive propre au template de chaque composant.                                                                |
| `totalScore`      | Somme des scores par dimension : flux du template, slots, prop drilling, état global, provide/inject, attributs fallthrough et graphe réactif. |
| `band`            | Catégorie lisible : `low`, `moderate`, `high` ou `extreme`.                                                                             |

L’entrée brute conserve aussi les chiffres derrière le score, notamment :

| Signal                                                            | Pourquoi c’est important                                                                                          |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `templateCyclomatic` et `templateCognitive`                       | Les scores propres des templates, additionnés sur tous les composants.                                            |
| `templateMaxNesting`                                              | L’imbrication la plus profonde de branches, de boucles et de slots à portée dans un même template.                |
| `templateScopedSlotCount`                                         | Les slots à portée couplent les templates parent et enfant ; ils sont donc comptés à part des slots ordinaires. |
| `templateUnknown`                                                 | Les expressions sans AST analysé (par exemple les handlers à plusieurs instructions). Elles n’ajoutent rien aux scores. |
| `propDrillingEdgeCount`                                           | Les arêtes de props indiquent un flux de données qui traverse les frontières.                                     |
| `provideInjectMaxDepth` et `provideInjectReferenceCount`          | Des arbres DI profonds ou larges rendent la propriété plus difficile à inspecter localement.                      |
| `reactiveNodeCount`, `reactiveEdgeCount` et `reactiveCycleCount`  | Les graphes réactifs capturent l’état au niveau des déclarations, les effets et les cycles propices aux pertes.    |

## Frontières des composants

La complexité du template a deux vues, toutes deux calculées à partir des mêmes faits :

- La complexité **propre** (own) ne regarde que le template du composant. La règle de lint
  `vue/max-template-complexity` juge cette vue : extraire une branche dans un composant enfant fait
  donc toujours baisser le score du parent.
- La complexité **rendue** (rendered) est le score propre du composant plus le score propre de chaque
  composant distinct qu’il rend, en suivant le graphe d’utilisation des composants que Croquis résout
  à travers les imports. Un enfant rendu depuis deux endroits compte une seule fois. Un composant
  récursif, et un groupe de composants qui se rendent mutuellement, comptent aussi une seule fois.

`CrossFileResult.templateComplexity` liste chaque composant avec les deux vues, l’arbre de rendu le
plus complexe en premier. Pour chaque composant, il donne aussi les constructions qui ajoutent de la
complexité, avec leur ligne et leur colonne.

Ainsi, un composant qui paraît peu profond peut quand même produire un score élevé lorsqu’il transmet
des slots à portée, fait du prop drilling ou dépend d’un chemin provide/inject profond. Le mode
Cross-file du Playground affiche le score à côté des diagnostics, de sorte que ces signaux restent
visibles pendant l’édition des fixtures.

## Règle de lint et constat Doctor

`vue/max-template-complexity` signale un `warning` lorsque le template propre d’un composant a une
complexité cyclomatique supérieure à 11 ou une complexité cognitive supérieure à 16. Ces limites sont
le p95 du corpus réel de Vize, soit 40 724 templates. L’avertissement pointe la balise `<template>` et
étiquette les cinq constructions qui ajoutent le plus de complexité.

Comme les limites sont un p95, environ un composant réel sur vingt les dépasse. Aucun preset n’active
la règle : l’activer est une décision propre au projet. Nommez-la sous `linter.rules` pour l’activer :

```ts
export default defineConfig({
  linter: {
    rules: {
      "vue/max-template-complexity": "warn",
    },
  },
});
```

`vize doctor` signale un point chaud de complexité de template, sous forme de notice, lorsque la
complexité rendue d’un composant dépasse le p95 du corpus : 106 en cyclomatique ou 139 en cognitive.

La complexité cyclomatique ajoute 1 par décision : chaque condition `v-if` / `v-else-if`, chaque
`v-for`, et chaque opérateur logique et `?:`. La complexité cognitive se compte ainsi :

- `v-if` et `v-for` ajoutent 1 plus leur profondeur d’imbrication.
- `v-else-if` et `v-else` ajoutent 1 chacun.
- Chaque suite de `&&`, `||` ou `??` ajoute 1.
- Un `?:` ajoute 1 plus sa profondeur d’imbrication.
- Le corps d’un slot à portée compte comme un niveau plus profond.

## Points chauds

Le rapport expose aussi des points chauds classés, pour que les outils puissent désigner les fichiers
et composants qui produisent le score au lieu de n’afficher qu’un seul chiffre pour tout le projet.
Chaque point chaud porte l’entrée locale du score, les scores par dimension, le score total et la
dimension dominante. Utilisez `dominantDimension` pour expliquer pourquoi l’entrée est élevée, puis
`input` pour montrer le signal brut qui l’a produite.

## Surface actuelle

La forme JSON publique est disponible via la liaison WASM d’analyse croisée de fichiers, sous
`CrossFileResult.complexityReport`, `CrossFileResult.complexityHotspots` et
`CrossFileResult.templateComplexity`. Le CLI ne fait pas encore échouer les builds sur ce score.
Utilisez le rapport comme signal exploratoire, puis ne promouvez des seuils stables qu’une fois des
références propres au projet établies.
