---
title: Complexité croisée de fichiers
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# Complexité croisée de fichiers

Le rapport de complexité croisée de fichiers de Vize est un résumé graphique-projet produit par Croquis. Ce n’est pas une règle diagnostique
en soi ; c’est un score explicable que les outils en aval peuvent afficher dans les rapports,
Playground et les futures vérifications basées sur des seuils.

Le modèle associe trois signaux de complexité à Vue :

- Nombre de chemins de modèle : un point de base par composant, plus `v-if`, `v-for`, et
  opérateurs booléens dans `v-if` expressions.
- Flux de contrôle imbriqué : un flux de modèles plus profond coûte plus cher, y compris le fait d’imbriquer
  se poursuit via des composants enfants.
- Flux de données composant-frontière : les arêtes props, supply/inject et réactives restent
  visibles comme des signaux transfrontaliers au lieu d’être aplatis en un seul fichier.

## Scores

Le rapport expose à la fois les signaux bruts et les scores dérivés.

| Terrain           | Signification                                                                                                                                     |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Nombre de bases des composants + `v-if` + `v-for` + opérateurs booléens dans `v-if`.                                                              |
| `cognitiveScore`  | Score de imbrication des modèles d’arbre de composants sur `v-if`, `v-for`, et emplacements de portée (scoped).                                   |
| `totalScore`      | Somme des scores dimensionnels : flux de modèles, fentes, forage de prop, état global, fourni/injecter, attraits à fallthrough et graphe réactif. |
| `band`            | Seau face à l’humain : `low`, `moderate`, `high`ou `extreme`.                                                                                     |

L’entrée brute conserve également les chiffres derrière le score, notamment :

| Signal                                                            | Pourquoi cela compte                                                                                                                                  |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `componentTreeVIfMaxDepth`                                        | Les longs chemins conditionnels entre les composants parent et enfant nécessitent plus d’états pour être testés.                                      |
| `componentTreeVForMaxDepth`                                       | Les boucles imbriquées sur les limites des composants amplifient la complexité du rendu et de la forme des données.                                   |
| `componentTreeScopedSlotMaxDepth`                                 | Les machines à sous à portée associent des modèles parent et enfant, de sorte que leur profondeur est suivie séparément du nombre habituel de fentes. |
| `propDrillingEdgeCount`                                           | Les arêtes de prop indiquent un flux de données transfrontalier.                                                                                      |
| `provideInjectMaxDepth` et `provideInjectReferenceCount`          | Les arbres DI profonds ou larges rendent la propriété locale plus difficile à inspecter.                                                              |
| `reactiveNodeCount`, `reactiveEdgeCount`, et `reactiveCycleCount` | Les graphes réactifs capturent l’état au niveau de déclaration, les effets et les cycles sujets à la perte.                                           |

## Frontières composantes

La complexité des modèles n’est pas limitée à un seul SFC. Croquis construit d’abord un registre de modules et un graphe d’utilisation des composants
, puis parcourt les arêtes des composants avec une protection de cycle. Un parent `v-if` autour d’un enfant, un parent
`v-for` autour d’un enfant, et un emplacement avec portée enfant contribuent tous au même arbre de composants
chemin de nidification.

Cela signifie qu’un composant peu profond peut tout de même produire un score élevé lorsqu’il avance des emplacements à portée de portée,
exerce des propulsions, ou dépend d’un chemin profond de fournisseur/injection. Le mode Cross-file de Playground affiche le score
à côté des diagnostics, de sorte que ces signaux sont visibles lors de la modification des luminaires.

## Points chauds

Le rapport expose également des points chauds classés afin que les outils puissent pointer vers les fichiers/composants qui créent le score
au lieu de ne montrer qu’un seul chiffre au niveau du projet. Chaque point chaud transporte l’entrée locale du score,
les scores dimensionnels, le score total et la dimension dominante. Utilisez `dominantDimension` pour expliquer pourquoi l’entrée de
est élevée, puis utilisez `input` pour montrer le signal brut qui l’a alimentée.

## Surface actuelle

La forme JSON publique est disponible via la liaison à fichier croisé WASM sous les
`CrossFileResult.complexityReport` et `CrossFileResult.complexityHotspots`. Le CLI ne fait pas défaut
s’appuie encore sur ce point. Utilisez le rapport comme signal exploratoire, puis promouvez des seuils stables
seulement après avoir existé des références spécifiques au projet.
