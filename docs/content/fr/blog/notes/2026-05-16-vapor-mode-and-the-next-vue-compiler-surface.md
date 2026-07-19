---
title: Mode vapeur
description: Pourquoi le mode Vapor est important pour Vize, et pourquoi un chemin de compilateur direct et précis change plus que la performance à l’exécution.
---

<!-- Generated translation; source: blog/notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface.md -->

# Mode vapeur

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

Le mode Vapor est facile à décrire trop étroitement.

La version courte est : afficher les composants Vue avec un chemin plus direct et plus précis et moins de surcharge du DOM virtuel.

C’est vrai, mais cela manque la question plus intéressante des outils.

Si le compilateur devient plus direct, alors la surface du compilateur devient plus importante.

## Pourquoi la vapeur est importante

Le rendu Vue traditionnel possède un modèle mental fort et mature :

- compiler des modèles en fonctions de rendu
- créer des nœuds virtuels
- Régions dynamiques différencielles
- patcher le DOM

Ce modèle est flexible et éprouvé au combat.

Vapor demande ce qui se passe lorsque le compilateur peut générer une représentation plus directe de l’interface réactive. Au lieu de traiter le DOM virtuel comme l’abstraction centrale à l’exécution, le compilateur peut émettre des opérations qui relient la réactivité par câble plus proches des mises à jour du DOM elles-mêmes.

Cela déplace la pression de la généralité à l’exécution vers la précision au moment de la compilation.

Pour Vize, c’est excitant car Vize repose déjà sur l’idée qu’une chaîne d’outils Vue doit comprendre en profondeur le SFC avant qu’il n’émette quoi que ce soit.

## Un type différent de responsabilité du compilateur

Lorsque la sortie du compilateur est plus directe, les erreurs deviennent plus marquées.

Le compilateur doit savoir :

- quelles liaisons sont réactives
- quelles opérations DOM sont stables
- quelles expressions nécessitent des obtenteurs
- quels props dynamiques nécessitent des chemins de mise à jour
- quels emplacements et composants nécessitent des limites d’exécution
- quels champs-clés de modèles sont locaux aux boucles, branches et emplacements

Dans un modèle DOM virtuel, une certaine incertitude peut être absorbée par diffing à l’exécution.

Dans un modèle plus direct de type Vapor, le compilateur porte davantage l’intention. Cela signifie que la qualité de l’analyse compte plus que tout. La cartographie des sources compte le plus. La couverture instantanée compte davantage.

C’est exactement le genre de problème que Vize est conçu pour explorer.

## Vapor en tant que backend de première classe

L’architecture de Vize considère les modes de sortie du compilateur comme des backends apparentés, et non comme des implémentations non apparentées.

La même structure SFC et l’analyse des modèles devraient pouvoir alimenter :

- Sortie du compilateur DOM
- Sortie du compilateur SSR
- Sortie du compilateur Vapor
- Diagnostics qui expliquent pourquoi une construction est prise en charge ou non

Cela compte car la vapeur ne doit pas devenir un cas spécial déconnecté.

Si le support Vapor se situe dans le même modèle de chaîne d’outils que le support DOM et SSR, Vize peut comparer les sorties, réutiliser des instantanés et rendre les diagnostics plus cohérents entre les modes.

## Changements de surface de débogage

Le mode Vapor modifie aussi l’expérience de débogage.

Lorsque la production est plus directe, les développeurs ont besoin de confiance en :

- Ordre des opérations générées
- Frontières de dépendance réactives
- Placement des auditeurs lors de l’événement
- Sémantique de mise à jour des props des composants
- Comportement de nettoyage des branches et boucles
- Hydratation ou compatibilité SSR lorsque c’est pertinent

Ce n’est pas seulement un problème d’exécution. C’est une question d’outillage.

Une bonne chaîne d’outils Vapor devrait aider à répondre :

- Qu’est-ce que le compilateur pensait être statique ?
- Qu’est-ce qu’il pensait être dynamique ?
- D’où vient un parcours de mise à jour particulier ?
- Quelle expression source a produit cette opération générée ?
- Pourquoi cette construction a-t-elle reculé ou échoué ?

C’est là que l’analyse statique et l’approche de tests axés sur les instantanés de Vize deviennent utiles.

## Performance sans perdre la sémantique

La vapeur est orientée performance, mais la performance ne peut pas se faire au détriment de la sémantique Vue.

Les utilisateurs ne devraient pas avoir à mémoriser un second langage modèle juste pour utiliser le chemin le plus rapide. Le meilleur résultat, c’est que le compilateur comprend suffisamment bien le code Vue pour que le rendu direct paraît naturel.

Cela nécessite :

- tests de compatibilité avec les attentes normales de Vue
- Matchs réels
- Diagnostics précis pour les motifs non pris en charge
- Cartographie des sources soigneusement réalisée
- des benchmarks incluant de grandes applications, pas seulement des exemples de jouets

L’objectif n’est pas « Vapor à tout prix ».

L’objectif est un chemin de compilateur rapide parce qu’il comprend davantage, pas parce qu’il supporte silencieusement moins.

## Pourquoi cela convient à Vize

Vize est encore expérimental. C’est précisément pour cela que la vapeur est un lieu naturel pour elle.

Une chaîne d’outils indépendante peut explorer :

- Formes alternatives de sortie du compilateur
- Diagnostics plus stricts
- Instantanés plus rapides
- Modélisation directe des opérations DOM
- Intégration avec l’analyse de modèles sensible aux types
- Explications orientées vers l’IA des choix des compilateurs

L’écosystème officiel a besoin de stabilité. Vize peut aller plus vite, tester de façon agressive et apprendre en public.

C’est la bonne relation.

Le mode Vapor n’est pas juste une case à cocher pour Vize. C’est un test de résistance pour toute l’idée d’une chaîne d’outils Vue unifiée.

Si l’analyseur, l’analyseur, le compilateur, les diagnostics, les instantanés et les éléments réels s’alignent tous, alors Vapor devient bien plus qu’une simple optimisation en temps réel.

Cela prouve que la chaîne d’outils comprend suffisamment Vue pour lui offrir un avenir différent.
