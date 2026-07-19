---
title: Carte d’outillage Vue
description: Une carte de la position de Vize dans le paysage actuel des outils de Vue, et de la façon dont il diffère des projets adjacents.
---

<!-- Generated translation; source: blog/notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape.md -->

# Carte d’outillage Vue

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publié le</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Auteur</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

Une des raisons pour lesquelles Vize est facile à mal comprendre est qu’il chevauche plusieurs outils que les gens connaissent déjà, mais pas toujours au même niveau.

Certains de ces projets sont officiels. Certains sont indépendants du framework. Certains sont centrés sur l’éditeur. Certains sont axés sur le compilateur en premier. D’autres concernent principalement la vérification des types. Certains essaient de devenir une véritable chaîne d’outils.

Donc la question la plus utile n’est pas « lequel est le meilleur ? » C’est : **quel problème chaque outil essaie-t-il réellement de résoudre ?**

## La version courte

Voici la façon la plus rapide de les positionner :

| Projet                       | Centre de gravité principal                                                                                                               | Ce qu’il n’est pas                                                                 |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| **Vize**                     | Chaîne d’outils complète indépendante Vue en Rust                                                                                         | Pas la pile officielle d’éditeurs Vue                                              |
| **Outils linguistiques Vue** | Éditeur officiel Vue + outils de vérification de type                                                                                     | Pas une chaîne d’outils complète de compilation/linter/formateur                   |
| **Golar**                    | `typescript-go`cadre de vérification des types en langage intégré                                                                         | Ce n’est pas une chaîne d’outils complète spécifique à Vue                         |
| **Verter**                   | Compilateur complet Vue alternatif + LSP + chaîne d’outils de compilation                                                                 | Ce n’est pas la chaîne d’outils officielle de Vue                                  |
| **Vite+**                    | Point d’entrée unifié du développement web à travers les runtimes, la gestion de paquets, le développement/construction/vérification/test | Pas un compilateur ou un linter spécifique à Vue                                   |
| **Oxlint**                   | Lintiere JS/TS haute performance                                                                                                          | Ce n’est pas une pile complète de peluches consciente des modèles Vue à elle seule |

Si vous gardez cette table en tête, la plupart de la confusion disparaît.

## Vize

Vize est mieux compris comme une **chaîne d’outils Vue indépendante et complète en Rust**.

Son ambition est large :

- compiler les SFC Vue
- Motifs spécifiques à la peluche Vue
- formater Vue
- vérification de type Vue et liaisons de script
- alimenter un LSP
- fournir une galerie de composants
- exposer des outils conscients de Vue aux flux de travail d’IA

Cette ampleur est ce qui distingue Vize de la plupart des projets de cette comparaison. Ce n’est pas seulement une intégration avec un éditeur, pas seulement un vérificateur de type, ni juste un plugin de bundler. Il cherche à être une chaîne d’outils cohérente et native de Vue, avec un centre architectural unique.

C’est aussi pourquoi la récente directive de contrôle de type est importante. Vize ne cherche pas seulement à « rendre `vue-tsc` plus rapide ». L’orientation actuelle est de garder la génération de fichiers virtuels, la cartographie de diagnostics et les informations de type vers l’éditeur conscientes de Vue, à l’intérieur de `vize_canon`, avec des sessions de projet natives alimentées par [`corsa-bind`](https://github.com/ubugeeei/corsa-bind).

## Comment Vize aborde-t-il `tsgo`

Une note récente, [`corsa-bind: The Idea of Language Processor Orchestration`](https://wtrclred.io/posts/17), soutient que la partie intéressante n’est pas seulement une exécution plus rapide, mais aussi de « changer la forme du travail, pas le compilateur ».

Ce cadre est très proche de la façon dont Vize aborde `tsgo`.

Vize ne cherche pas à transformer `tsgo` en toute l’histoire du produit, ni ne le traite pas comme un coup de ligne de crédit one-shot qui est relancé pour chaque fonctionnalité. Cette orientation se rapproche davantage du traitement de TypeScript comme un service natif réutilisable au sein d’une chaîne d’outils Vue plus large :

- `vize check` matérialise un projet TypeScript virtuel conscient de Vue, ouvre une session de projet Corsa et demande des diagnostics par lot.
- `vize_maestro` peut garder un pont Corsa pour le survol, la complétion, la définition, les références, et renommer lorsque la vérification de type native est activée.
- `vize_patina` utilise des sessions Corsa natives paresseuses pour des règles de lint sensibles aux types, ne sondant que les types dont il a besoin au lieu de tout reconstruire dans une pile hébergée en JavaScript.
- `vize_canon` conserve la propriété de la génération de fichiers virtuels spécifiques à Vue et du mappage source, tandis que `corsa-bind` et `tsgo` répondent aux questions côté TypeScript.

Ainsi, l’histoire `tsgo` de Vize n’est pas simplement « remplacer `vue-tsc` par un binaire plus rapide ». Il s’agit plutôt de construire une couche de contrôle native Vue autour d’un processeur TypeScript résident, puis de réutiliser cette couche à travers des tests batch, des fonctionnalités d’éditeur et un linting sensible au type.

## Outils linguistiques Vize vs Vue

Le projet officiel [Vue Language Tools](https://github.com/vuejs/language-tools) est l’éditeur Vue prêt à la production et la pile de vérification de typographie. Elle comprend :

- l’extension **Vue (Officiel)** VS Code
- `vue-tsc`
- `@vue/language-server`
- `@vue/language-core`

Cette pile concerne **fondamentalement les outils de langage** : support de l’éditeur, vérification de types, génération virtuelle de code et intégrations qui donnent à Vue une apparence de premier ordre dans les IDEs.

Vize chevauche ce monde car Vize possède aussi un vérificateur de type et un LSP. Mais Vize essaie de couvrir davantage de terrain :

- Vize inclut ses propres ambitions de compilateur
- Vize inclut des ambitions de linting et de mise en forme
- Vize inclut des surfaces de produit comme les outils Musea et MCP
- Vize est Rust-first plutôt que TypeScript-first

La distinction la plus simple est donc :

- **Vue Language Tools** est l’éditeur officiel et la fondation de vérification de typographie pour Vue
- **Vize** est une tentative indépendante d’unifier une grande partie de la chaîne d’outils Vue sous une seule architecture Rust

Si votre priorité est le support de l’éditeur prêt pour la production aujourd’hui, la pile officielle Vue est la base. Si votre intérêt porte sur une chaîne d’outils Vue plus large, expérimentale et native de Rust, c’est là que Vize commence à avoir du sens.

## Vize contre Golar

[Golar](https://github.com/auvred/golar) n’est pas vraiment « une autre chaîne d’outils Vue » au même sens.

Golar se décrit comme un cadre de langage embarqué basé sur `typescript-go`. Pour Vue en particulier, il réutilise la machinerie officielle `@vue/language-core` et se concentre sur la création de langages basés sur des extensions comme `.vue`, `.astro`et `.svelte` compatibles avec `tsgo`.

Cela signifie que le centre de gravité de Golar est :

- Vérification du type CLI
- déclaration émettrice
- `tsgo` intégration pour les langages embarqués
- Infrastructure de plugin pour la génération de code virtuel

Vize se distingue à deux égards importants :

1. **Portée**

Golar est principalement une histoire de vérification de typographie et de code virtuel autour de `typescript-go`.
Vize essaie de posséder une part bien plus large de la chaîne d’outils Vue : compilateur, linter, formateur, vérificateur de type, LSP, galerie, et bien plus encore.

2. **Propriété de la couche Vue**

Golar réutilise délibérément les outils officiels de Vue pour la génération de code Vue.
Vize essaie de construire davantage la stack spécifique à Vue elle-même dans Rust.

Il y a aussi une différence pratique dans la couche d’exécution qui commence à apparaître. Golar est étroitement associé à l’intégration `typescript-go` pour les langages embarqués. Le chemin de contrôle de type natif actuel de Vize est conçu autour de `vize_canon` plus `corsa-bind`, ce qui rend la question moins « comment réutiliser la pile officielle avec un moteur TS plus rapide ? » et plus « quelle part de la chaîne d’outils Vue peut résider dans une seule architecture native ? »

Ainsi, Golar est plus proche de « faire fonctionner `tsgo` bien pour les langages embarqués », tandis que Vize est plus proche de « construire une chaîne d’outils Vue native de bout en bout ».

## Vize vs Verter

[Verter](https://github.com/pikax/verter) est probablement le voisin philosophique le plus proche de cette liste.

Comme Vize, Verter vise haut. Sa vision publique est un hybride Rust + TypeScript Vue compilateur, LSP, outil de compilation, linter et une chaîne d’outils plus large. Cela le place dans la même famille générale que Vize : ambitieux, full-stack, et prêt à repenser la chaîne d’outils Vue au lieu de ne patcher qu’une seule couche.

C’est là que les différences deviennent plus liées à la forme et à l’architecture du produit qu’à la catégorie :

- **Verter** se présente comme un langage Vue strictement d’abord et une chaîne d’outils de compilation, avec une forte histoire de fournisseur VS Code et TS.
- **Vize** se présente comme une chaîne d’outils Vue indépendante et haute performance avec une interface de ligne de ligne unifiée, une intégration Vite, Musea, et un récit plus fort de « un parseur / un AST / une seule chaîne d’outils ».

Il y a aussi une différence d’accent :

- Verter met en avant la génération TSX tapée, les backends de fournisseurs de types tels que TSGO / tsserver, ainsi qu’un large catalogue intégré de règles de lint.
- Vize met en avant une chaîne d’outils unifiée native Rust à travers compilation, lint, formatage, vérification de type, outils d’éditeur, galerie de composants et intégration IA, tout en se positionnant explicitement comme complémentaire aux outils de l’écosystème comme Vite+ et Oxlint.

Je ne décrirais donc pas Verter comme « la même chose avec un autre nom ». Il vaut mieux la considérer comme **une autre réponse sérieuse à la question : à quoi ressemblerait une chaîne d’outils Vue de nouvelle génération si nous recommencions ?**

## Vize vs Vite+

[Vite+](https://viteplus.dev/) se situe à une autre couche.

Vite+ est un point d’entrée unifié pour le développement web de manière plus générale. Sa mission est de gérer la configuration à l’exécution, la gestion des paquets, le développement, la vérification, les tests, la compilation, l’emballage et l’exécution des tâches monodépôt dans un seul flux de travail. Il rassemble Vite, Vitest, Oxlint, Oxfmt, Rolldown, tsdown et les outillages associés.

Cela fait Vite+ :

- **Indépendant du cadre**
- Orienté flux de travail
- plus large que Vue

Vize est différent parce qu’il **est spécifique à Vue**.

Vite+ ne cherche pas à devenir un compilateur Vue ou un linter de modèles Vue. Cela vous offre un point d’entrée unifié dans la chaîne d’outils web.
Vize peut se connecter à ce monde. En fait, ce dépôt utilise déjà Vite+ pour l’orchestration des espaces de travail.

Donc ce n’est pas vraiment une compétition :

- **Vite+** = le shell général de la chaîne d’outils web
- **Vize** = le moteur spécifique Vue qui peut vivre à l’intérieur de cette coque

## Vize contre Oxlint

[Oxlint](https://oxc.rs/docs/guide/usage/linter) est aussi à un niveau différent.

Oxlint est le linter haute performance JavaScript et TypeScript de l’écosystème Oxc. Il est excellent pour les règles générales JS/TS et les flux de travail de plus en plus sensibles aux types, mais il n’est pas destiné à remplacer tous les diagnostics compatibles avec les modèles Vue.

C’est là que Vize Patina entre en jeu.

Patina se concentre sur des questions spécifiques à la linting de Vue, telles que :

- Directives modèles
- Structure SFC
- Conventions des composants
- Contrôles d’accessibilité dans les modèles Vue

La différence est donc simple :

- **Oxlint** gère le linting JS/TS polyvalent
- **Vize / Patina** gère la linting spécifique à Vue

Le nouveau `oxlint-plugin-vize` alpha existe précisément parce que ces deux éléments sont complémentaires plutôt que redondants.

## Alors, où se situe Vize ?

Vize se situe dans le chevauchement de plusieurs catégories, mais il n’est pas réductible à aucune d’entre elles.

C’est :

- plus large que les outils linguistiques officiels de Vue
- plus large que `tsgo` des projets d’accélération comme Golar
- le plus proche en ambition des efforts alternatifs full-stack comme Verter
- complémentaire aux outils de flux de travail généraux comme Vite+
- complémentaire aux linters JS/TS généraux comme Oxlint

Si je devais le condenser en une seule phrase :

> Vize est une tentative indépendante native de Rust visant à unifier bien plus la chaîne d’outils Vue que ce que couvrent les outils officiels du langage, tout en coopérant avec des outils plus larges de l’écosystème plutôt que de les remplacer.

## Lequel devriez-vous choisir ?

Cela dépend de ce que vous voulez :

- Choisissez **Vue Language Tools** si vous souhaitez dès aujourd’hui la pile officielle d’éditeur et de vérification de typographie prête pour la production.
- Regardez **Golar** si votre principal intérêt est la vérification des types basée sur `typescript-go`pour les langages intégrés tout en réutilisant les outils des langues officielles.
- Regardez **Verter** si vous voulez une autre chaîne d’outils Vue full-stack ambitieuse avec un typage strict et une histoire LSP solide.
- Utilisez **Vite+** si vous voulez un point d’entrée unifié pour le développement web et polyvalent.
- Utilisez **Oxlint** si vous avez besoin de JavaScript haute performance et de linting TypeScript.
- Utilisez **Vize** si ce qui vous enthousiasme, c’est la possibilité d’une chaîne d’outils Vue native Rust plus large, qui essaie de faire en sorte que le compilateur, le linting, la mise en forme, la vérification de type, les outils d’éditeur, les outils de galerie et les outils d’IA ressemblent à un seul système.

C’est la vraie différence.
