---
title: Comparaison des outillages
description: Une comparaison pratique de Vize et des projets voisins à travers les outillages officiels Vue, Oxc, Golar, Verter, Flint et TSSLint.
---

<!-- Generated translation; source: blog/notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint.md -->

# Comparaison des outillages

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publié le</span>
<span class="blog-meta-value">16 janvier 2026</span>
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

Vize est suffisamment proche de plusieurs projets pour que la comparaison soit inévitable.

Cette comparaison est utile, mais seulement si l’axe est clair. « Plus vite » ne suffit pas. « Rouille » ne suffit pas. Le « soutien Vue » ne suffit pas.

La vraie question est : **quelle couche chaque projet souhaite-t-elle posséder ?**

![Relationship map showing Vize in the nearby tooling landscape, with reference-only, adjacent platform, used-by-Vize, and compare-only groups](/blog/vize-toolchain-map.svg)

## Carte rapide

| Projet                      | Centre de gravité                                                      | Comment Vize s’y rapporte-t-il                                                                    |
| --------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Outillages officiels de Vue | La base de production pour les outils du compilateur et du langage Vue | Vize est indépendant et expérimental, il doit donc considérer cela comme point de référence       |
| Oxc / Oxlint                | Infrastructure générale JavaScript et TypeScript                       | Vize peut réutiliser et coopérer avec Oxc tout en possédant une sémantique spécifique à Vue.      |
| Golar                       | `typescript-go`vérification des types en langage intégré               | Vize propose une gamme d’outils Vue plus large que la vérification de type seule                  |
| Verter                      | Compilateur et chaîne d’outils Vue de nouvelle génération alternatif   | Plus proche en ambition, différent en architecture et en forme de produit                         |
| Flint                       | Linting JS/TS amical et typé avec des valeurs par défaut fortes        | Complémentaire pour le linting TS général, pas une chaîne d’outils SFC Vue                        |
| TSSLint                     | Linting natif TypeScript à l’intérieur du serveur de langue            | Idée de linting sémantique solide, mais pas une pile complète de compilateurs/linter/galeries Vue |

## Outillages officiels de Vue

La pile officielle compte d’abord.

[Vue Language Tools](https://github.com/vuejs/language-tools), `vue-tsc`, les paquets compilateurs Vue et les intégrations officielles des éditeurs constituent la base de production. Lorsque Vize est en désaccord avec le comportement officiel, ce désaccord n’est pas automatiquement une idée audacieuse. La plupart du temps, c’est une correction nécessaire, une implémentation incomplète, ou un point où Vize a besoin d’une histoire de compatibilité plus claire.

Cela ne rend pas Vize inutile.

Il définit le contrat.

Vize peut expérimenter une architecture native Rust plus unifiée, mais il doit toujours se soucier de la forme du vrai code Vue, de la sortie réelle du compilateur, des vrais diagnostics et des attentes réelles de l’éditeur. La pile officielle est le point de référence qui maintient l’expérience honnête.

## Oxc et Oxlint

[Oxc](https://oxc.rs/) est un projet d’infrastructure compilateur JavaScript et TypeScript à usage général. [Oxlint](https://oxc.rs/docs/guide/usage/linter.html)'est le linter haute performance construit sur ce monde.

Vize ne devrait pas concurrencer Oxc au niveau JavaScript et TypeScript. Ce serait du gaspillage. Oxc offre déjà à l’écosystème un analyseur rapide, une infrastructure sémantique, une direction de formatage, une direction linter, et un ensemble croissant de primitives partagées.

La question Vize est plus étroite et plus spécifique à Vue :

- Qu’est-ce qu’un fichier `.vue` dans son ensemble ?
- Comment les longueurs de portée des modèles se connectent-ils aux liaisons de script ?
- Comment les directives, les emplacements, les props, les emits, les blocs de style et la sortie du compilateur se rapportent-ils ?
- Comment redistribuer les diagnostics à la source exacte que les humains modifient ?
- Comment ces flux sémantiques compilant-ils, lint, formatez-vous, vérifient-ils les types, LSP, Musea et les flux de travail IA ?

L’oxc peut être la base générale JS/TS. Vize peut être la chaîne d’outils spécifique à Vue qui utilise cette base sans aplatir Vue en « de simples blocs de script ».

## Golar

[Golar](https://github.com/auvred/golar) est intéressant car il prend `typescript-go` sérieux pour les langages embarqués.

Son cœur est la vérification de type, le code virtuel et l’intégration `tsgo` . Pour Vue, cela le place naturellement proche du modèle officiel de base linguistique. C’est une bonne et pratique forme : réutiliser la machine de code virtuel de Vue et rendre le moteur TypeScript plus rapide ou plus flexible.

Vize essaie de résoudre un problème plus large.

La couche de vérification de type compte, mais ce n’est pas l’ensemble du projet. Vize souhaite que l’analyseur, le modèle sémantique, le compilateur, le linter, le formateur, le chemin de vérification de type natif, le LSP, la galerie de composants et les surfaces orientées IA partagent davantage du même cœur conscient de Vue.

Donc la différence n’est pas « Golar vérifie le type et Vize est plus rapide ».

La différence est la suivante :

- Golar est principalement une histoire de traitement TypeScript en langage intégré.
- Vize est une histoire complète de la chaîne d’outils Vue où la vérification de type est un des utilisateurs du modèle d’analyse Vue.

## Verter

[Verter](https://github.com/pikax/verter) est probablement la comparaison la plus proche philosophiquement.

Il pose aussi une grande question : à quoi ressemblerait une chaîne d’outils Vue de nouvelle génération si nous étions prêts à repenser les couches ?

C’est proche de la question de Vize. Les deux projets se soucient du comportement des compilateurs, des outils du langage, du diagnostic, et d’une expérience plus stricte que ce qu’un simple ensemble de plugins sans lien peut offrir.

Les différences résident dans l’emphase :

- Verter apparaît plus strict et orienté vers le service linguistique dès le début.
- Vize met l’accent sur un noyau partagé Rust-native à travers les flux de travail de compilation, de lint, de formatage, de vérification, de LSP, de Musea et d’IA.
- Vize considère également les outils de galerie de composants et de systèmes de conception comme des éléments de première classe de l’environnement frontend, et non comme des éléments documentaires séparés après coup.

Je ne considère pas Verter comme un ennemi. C’est une autre expérience sérieuse dans un domaine qui mérite plusieurs expériences.

## Flint

[Flint](https://www.flint.fyi/) est une comparaison différente.

C’est un linter JavaScript et TypeScript mettant l’accent sur les paramètres par défaut utiles, la mise en cache et le linting typé. C’est précieux car l’écosystème JS/TS pose un vrai problème : le linting uniquement syntaxique est rapide mais incomplet, tandis que le linting sémantique peut devenir lent et coûteux opérationnellement.

Vize est d’accord avec le principe que le retour sémantique doit être pratique, rapide et agréable.

Mais Flint ne cherche pas à être un compilateur SFC Vue, formateur, analyseur de modèles, galerie de composants ou LSP spécifique à Vue. Il est mieux compris comme une direction générale de linting de haute qualité.

La forme complémentaire est :

- Flint peut faire avancer l’expérience du linting JS/TS.
- Vize peut faire avancer l’analyse spécifique à Vue.
- Un bon environnement frontend devrait permettre à ces couches de coopérer au lieu de forcer chaque outil à assumer chaque préoccupation.

## TSSLint

[TSSLint](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.vscode-tsslint) est important car il traite le linting sémantique de TypeScript comme quelque chose pouvant être proche du serveur du langage TypeScript.

Cette idée est convaincante : si le vérificateur TypeScript a déjà un projet ouvert, pourquoi reconstruire le monde dans un processus linter séparé juste pour répondre à des questions sémantiques ?

Vize a un instinct similaire, mais pointant vers Vue comme un artefact multilingue.

Pour Vize, la question n’est pas seulement « les règles de peluches peuvent-elles réutiliser l’état de TypeScript ? » C’est :

- L’analyse de modèles peut-elle réutiliser le même modèle sémantique Vue que le compilateur ?
- Les règles de peluches Vue sensibles au type peuvent-elles poser des questions ciblées sans payer le coût total de la reconstruction ?
- Les diagnostics de l’éditeur, les vérifications par lots et les boucles de réparation IA peuvent-ils s’accorder sur la même correspondance source ?
- Le système peut-il maintenir une session de projet assez longtemps pour amortir le travail ?

TSSLint est un signal fort indiquant que le linting sémantique souhaite se rapprocher de l’état linguistique existant. Vize étend cet instinct dans la structure spécifique à Vue.

## Ce que Vize essaie de posséder

Vize ne devrait pas tout posséder.

Il devrait posséder les domaines où la connaissance spécifique à Vue doit être cohérente :

- Analyse syntaxique SFC et structure de blocs
- Sémantique des modèles
- Analyse directive et composante
- Décisions de sortie du compilateur
- Diagnostic des peluches conscient de Vue
- Cartographie source des artefacts générés vers `.vue`
- Métadonnées composantes pour Musea
- Diagnostics lisibles par machine pour les flux de travail d’IA

Elle devrait coopérer ailleurs :

- utiliser Oxc pour JavaScript et l’analyse TypeScript lorsque possible
- comparer le comportement avec les outils officiels de Vue
- apprenez de Golar, TSSLint et Flint sur des boucles de rétroaction sensibles au type
- Restez conscients de Verter comme d’une autre expérience de chaîne d’outils complète

## La position du produit

Le positionnement le plus net est le suivant :

> Vize est une chaîne d’outils Vue indépendante, expérimentale, native de Rust, qui tente de faire en sorte que le compilateur, linter, formateur, vérificateur de types, LSP, galerie de composants et diagnostics destinés à l’IA ressemblent à un environnement cohérent.

Cela signifie que Vize n’est pas la réponse officielle.

C’est une réponse expérimentale à grande vitesse.

Le travail maintenant est de rendre cette réponse utile dans de vrais projets, de réduire l’écart avec le comportement officiel, et de garder l’architecture suffisamment nette pour que l’expérience vaille la peine d’être menée.
