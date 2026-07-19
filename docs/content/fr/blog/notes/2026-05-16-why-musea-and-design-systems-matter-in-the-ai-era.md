---
title: Musea & IA
description: L’IA peut générer rapidement une interface utilisateur, mais Musea et les systèmes de conception rendent l’intention, les contraintes, l’accessibilité et le flux de travail de relecture durables.
---

<!-- Generated translation; source: blog/notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era.md -->

# Musea & IA

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

![Musea home view showing a design-system gallery surface](/musea-home.png)

L’IA rend la production d’interface peu coûteuse.

C’est utile, mais cela change aussi le goulot d’étranglement. La partie difficile n’est plus seulement « peut-on fabriquer un composant ? » Le plus difficile est :

- Est-ce que ça correspond au produit ?
- Est-ce que cela respecte le système de conception ?
- Est-ce accessible ?
- Est-ce cohérent avec les États existants ?
- Les évaluateurs peuvent-ils comprendre ce changement ?
- Les futurs agents peuvent-ils réutiliser la même intention ?

C’est pourquoi Musea compte.

## Contraintes des besoins de production

Un modèle d’IA peut produire cinq versions d’un composant en quelques secondes.

Mais sans contraintes, ces versions dérivent :

- Modifications de l’espacement
- Les États manquent
- Les couleurs sont proches mais non tokenisées
- L’accessibilité est considérée comme une suggestion
- Les états vide, de chargement, d’erreur et désactivés sont oubliés
- La hiérarchie visuelle change sans décision de conception

Le système de conception est la couche de contrainte.

Il indique aux humains et aux agents ce que signifie « bon » pour ce produit.

## Les systèmes de conception doivent devenir exécutables

Un système de conception ne peut pas être seulement une page Figma, un README ou un accord tribal.

Dans un flux de travail axé sur l’IA, l’intention de conception doit être lisible par machine :

- jetons
- Métadonnées des composants
- Exemples
- États
- Attentes en matière d’accessibilité
- Lignes de base de régression visuelle
- Notes d’utilisation
- Documents générés

C’est la direction que prend Musea.

Musea n’est pas qu’une galerie. C’est un moyen de faire de la surface du système de conception une partie de la chaîne d’outils.

![Musea token view showing design tokens as a concrete product surface](/musea-tokens.png)

## Ce que Musea essaie d’offrir

Les caractéristiques pratiques comptent :

- Pages de galeries composantes
- Fichiers d’art qui décrivent des exemples et des états
- Documentation générée
- Flux de travail palette et jetons
- Contrôles d’accessibilité
- Test de régression visuelle
- Intégration Vite pour l’exploration locale
- Intégration MCP afin que les outils d’IA puissent inspecter le contexte des composants

Le but n’est pas de faire un catalogue plus joli.

L’objectif est de transformer les composants en artefacts vérifiables, vérifiables et documentés.

Lorsqu’un agent change un composant, Musea devrait aider à répondre :

- Quels États ont changé ?
- Quels exemples sont affectés ?
- La base visuelle a-t-elle bougé ?
- L’accessibilité a-t-elle régressé ?
- Le composant correspond-il toujours à son intention documentée ?
- Un autre agent peut-il comprendre comment l’utiliser ?

## L’IA a besoin de mémoire produit

Les mannequins ne connaissent pas automatiquement votre produit.

Ils connaissent peut-être les schémas généraux de l’interface, mais la qualité du produit réside dans les détails :

- quel ton utilise l’interface utilisateur
- À quel point les écrans opérationnels devraient être denses
- quels contrôles sont canoniques
- Comment les actions destructrices sont présentées
- Comment se comportent les États vides
- Comment gérer les compromis entre la marque et l’accessibilité

Le musea peut devenir une mémoire produit pour ces détails.

Il offre aux flux de travail de l’IA quelque chose de mieux qu’une simple invite : une surface structurée composée de composants réels, d’états réels, d’exemples réels et de contraintes réelles.

## La revue visuelle devient plus importante

Une interface générée par l’IA peut sembler plausible tout en restant fausse.

La disposition peut être subtilement incohérente. Le contraste peut échouer. L’état de survol peut modifier la disposition. Une longue étiquette peut mal s’enrouler. Un état de chargement peut couvrir un contexte important.

C’est pourquoi le test de régression visuelle appartient à la galerie des composants.

L’analyse statique peut détecter des erreurs structurelles. Le contrôle du type peut détecter les contrats. Mais les systèmes visuels ont besoin de preuves visibles.

Musea devrait rendre la revue visuelle une routine :

- Générer des états
- Capturer des captures d’écran
- Comparer les références
- Diffs de surface
- Gardez la critique proche du composant

Cela transforme la qualité du design en un flux de travail répétable au lieu d’un fil de capture d’écran de dernière minute.

## Les systèmes de conception sont des infrastructures d’IA

À l’époque pré-IA, un système de conception aidait surtout les humains à avancer plus vite et avec constance.

À l’ère de l’IA, elle aide aussi les machines à se déplacer en toute sécurité.

Un système de conception solide offre aux agents :

- Un vocabulaire
- Exemples à imiter
- contraintes à respecter
- Tests à réussir
- Documents à lire
- Références visuelles à préserver

C’est l’infrastructure.

Musea existe parce que Vize ne devrait pas s’arrêter à la correction du code. La qualité du frontend inclut la qualité visuelle, l’accessibilité et la cohérence du produit.

L’IA augmente le besoin de tout cela.

L’avenir n’est pas « l’IA génère une interface utilisateur, donc les systèmes de conception comptent moins ».

L’avenir est « l’IA génère une interface utilisateur, donc les systèmes de conception doivent devenir exécutables, inspectables et testables. »
