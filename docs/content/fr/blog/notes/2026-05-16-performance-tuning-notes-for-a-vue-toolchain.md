---
title: Réglage des performances
description: Leçons pratiques de performance issues de la construction d’une chaîne d’outils Vue où l’analyse syntatique, l’allocation, le parallélisme et les boucles de rétroaction comptent tous.
---

<!-- Generated translation; source: blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain.md -->

# Réglage des performances

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

Le réglage des performances dans une chaîne d’outils frontend n’est pas une seule astuce.

Ce n’est pas « réécrire dans Rust » puis attendre que les graphiques apparaissent. C’est une longue série de petites décisions concrètes sur la direction du temps, la fréquence de déplacement de la mémoire, la quantité de travail dupliquée, et si l’architecture permet de s’accumuler les améliorations.

Cette note partage les connaissances sur les choses pour lesquelles Vize continue d’optimiser.

![Feedback loop diagram showing source files, native analysis, snapshots, actions, and shipping confidence](/blog/feedback-loop.svg)

## Mesurez toute la boucle

Les benchmarks de compilateur sont utiles, mais ils ne constituent pas toute l’expérience développeur.

Une chaîne d’outils Vue comporte plusieurs boucles de rétroaction :

- Compilation en un seul fichier dans un serveur de développement
- Production en production complète
- Linting de nombreux dossiers
- formatage de nombreux fichiers
- vérification de type des fichiers virtuels générés
- Diagnostic de l’éditeur pendant que l’utilisateur tape
- Contrôles d’IC à travers des applications réelles
- Les correctifs générés par l’IA sont validés à plusieurs reprises

La boucle la plus lente n’est pas toujours la plus évidente.

Une fonction qui semble rapide isolément peut toujours être nuisible si elle fonctionne à chaque niveau. Une petite allocation peut toujours compter si cela se produit pour chaque jeton, chaque nœud AST, chaque diagnostic et chaque segment généré.

C’est pourquoi Vize considère la performance comme une propriété de chaîne d’outils, et non seulement comme une propriété de compilateur.

## Éviter les doublons

L’optimisation la plus fiable est de ne pas refaire le travail deux fois.

Dans une configuration fragmentée, le même fichier `.vue` peut être analysé séparément par :

- Le compilateur
- Le Linter
- Le Formateur
- Le vérificateur de caractères
- L’intégration de l’éditeur
- Le pipeline de documentation des composants

C’est coûteux, mais le problème le plus profond est architectural. Si chaque outil construit sa propre compréhension du fichier, l’ajustement des performances devient local et limité.

Vize est conçu autour d’une structure partagée :

- Analyser une fois lorsque c’est possible
- Maintenir les limites des blocs SFC stables
- Réutilisation de la structure du modèle à travers le compilateur et le diagnostic
- Laissez l’analyse sémantique nourrir plusieurs consommateurs
- évitez de régénérer le TypeScript virtuel sauf si les entrées changent

La meilleure optimisation est souvent une meilleure limite de propriété.

## L’allocation est une caractéristique, pas un détail

Les outils frontend traitent de nombreux petits objets : jetons, nœuds, spans, chaînes, portées, diagnostics, fragments de code générés.

Si ces objets sont alloués de manière informelle, la chaîne d’outils les paie partout.

Vize exerce beaucoup de pression sur le comportement d’allocation :

- stockage de type arena pour les données de compilateur de courte durée
- Internement de chaînes où des identifiants ou noms répétés sont importants
- Spans compacts au lieu de sous-chaînes copiées
- Parts empruntées où la propriété est inutile
- identifiants internes stables au lieu de grandes structures clonées

Le but n’est pas de rendre le code intelligent pour lui-même.

L’objectif est de rendre le chemin chaud ennuyeux : moins d’allocations, moins de copies, moins de manquements de cache, moins de raisons pour que l’allocateur fasse partie du profil.

## Le parallélisme a besoin de forme

Le parallélisme n’est pas « allumer des fils ».

Cela marche mieux lorsque le problème a des limites claires :

- de nombreux fichiers indépendants
- Agrégation déterministe
- Ordre de sortie prévisible
- Pas de mutation globale partagée
- Caches et sessions bornées

La compilation, le linting et les balayages de fixture Vue ont souvent une forme parallèle naturelle au niveau du lime. Mais la vérification de type et les flux de travail de l’éditeur sont plus subtils car ils dépendent de l’état du projet.

Ainsi, Vize sépare les questions :

- Ce travail au niveau du fichier peut-il s’exécuter de manière indépendante ?
- Cette étape nécessite-t-elle une session de projet résident ?
- L’ordre de sortie est-il visible pour l’utilisateur ?
- Les diagnostics sont-ils stables sur le nombre de threads ?
- Le parallélisme augmente-t-il suffisamment la pression mémoire pour effacer la victoire ?

Une sortie rapide mais instable n’est pas suffisante. Le travail de performance doit préserver la confiance.

## La cartographie source peut devenir une voie chaude

Les outils Vue génèrent souvent du code intermédiaire.

Cela signifie que tout bon diagnostic a besoin d’un chemin de retour :

- TypeScript généré vers le modèle original
- Code de rendu généré vers la source SFC
- sortie de style ou de script transformée vers le bloc d’origine
- identifiants de modules virtuels pour revenir aux fichiers réels

Si la cartographie des sources est lente ou imprécise, toute la chaîne d’outils en souffre. L’utilisateur voit les diagnostics au mauvais endroit. Les boucles de réparation de l’IA obtiennent de mauvaises coordonnées. Les tests deviennent fragiles.

Ainsi, le mappage source mérite la même attention en termes de performance que l’analyse syntaxique :

- S’étend de stockage de façon compacte
- éviter la normalisation des chemins répétés
- Gardez les métadonnées de segments générées petites
- Cas limites de test avec instantanés
- Charges de travail très axées sur le diagnostic de profil, pas seulement des chemins de compilation réussis

Le diagnostic est une surface produit. Leur performance compte.

## Les projets réels l’emportent sur le confort synthétique

Les microbenchmarks sont utiles pour répondre à une question ciblée.

Mais une chaîne d’outils devient honnête lorsqu’elle est gérée par rapport à de vrais projets.

Les projets réels comprennent :

- Dispositions de dépendances étranges
- Grands SFC
- Motifs hérités
- Code généré automatiquement
- Directives peu communes
- Conventions des plugins
- Aliases de chemin
- Cas particuliers spécifiques à chaque plateforme

C’est pourquoi Vize continue d’investir dans des balayages réels de fixations et des snapshots de construction. Le but n’est pas de collecter des résultats impressionnants aux tests. L’objectif est d’exposer les cliffs de performance qui n’apparaissent que lorsque le code est désordonné, de la même manière que le code de production l’est.

## La performance est une caractéristique du produit

La vitesse modifie le comportement.

Si les contrôles sont lents, les gens les font moins souvent.
Si la mise en forme est lente, sauvegarder sur le format devient agaçant.
Si le linting conscient du type est lent, les équipes désactivent les règles.
Si l’IC est lent, les mainteneurs font des lots de modifications et examinent moins attentivement.
Si la validation par IA est lente, les agents font des sauts plus importants et plus risqués.

Des outils rapides rendent pratiques des flux de travail plus stricts.

C’est là le véritable argument de performance pour Vize. L’objectif n’est pas seulement un meilleur chiffre de référence. Le but est de faire en sorte que le chemin strict ressemble à celui par défaut.

Lorsque la compilation, la charpente, la mise en forme, la vérification de type et le diagnostic deviennent suffisamment rapides pour s’exécuter sans cérémonie, la qualité cesse d’être un événement spécial.

C’est devenu la façon normale de travailler.
