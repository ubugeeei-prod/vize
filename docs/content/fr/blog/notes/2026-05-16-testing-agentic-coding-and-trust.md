---
title: Tests & Agents
description: Pourquoi les tests axés sur les instantanés, les fixatures réelles et les vérifications déterministes sont plus importantes lorsque les agents font partie de la boucle de développement.
---

<!-- Generated translation; source: blog/notes/2026-05-16-testing-agentic-coding-and-trust.md -->

# Tests & Agents

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

Le codage agent change le rôle des tests.

Quand un humain écrit un petit patch, les tests nous indiquent si le patch a cassé quelque chose.

Lorsqu’un agent peut réécrire de gros morceaux de code, les tests deviennent aussi le langage que nous utilisons pour indiquer à l’agent ce que signifie « bon ».

Cela rend les tests plus importants, pas moins.

## Les tests sont la mémoire du projet

Les agents sont bons pour raisonner localement, mais un projet est plus grand que le prompt actuel.

Une chaîne d’outils a accumulé des décisions :

- Que devraient dire les diagnostics
- où les portées de source doivent pointer
- À quoi devrait ressembler le code généré
- quels cas périphériques Vue sont pris en charge
- quels projets réels doivent continuer à compiler
- lesquels les faux positifs sont inacceptables

Les tests conservent ces décisions.

Sans tests, chaque changement agent est contraint de redécouvrir le projet à partir de zéro. Avec des tests, le projet peut résister. Elle peut dire : ce comportement compte, cette sortie est intentionnelle, ce message d’erreur fait partie de l’expérience utilisateur.

## Les tests d’instantanés sont particulièrement utiles

Vize utilise beaucoup d’instantanés car les chaînes d’outils produisent des résultats structurés que les humains doivent inspecter :

- Sortie du compilateur
- Sortie de formateur
- Diagnostic du linter
- TypeScript virtuel
- Emplacements de diagnostic mappés par source
- Métadonnées de Musea générées
- Construire des artefacts issus de projets de fixture

Les instantanés ne remplacent pas les assertions. Ils sont un moyen de rendre un comportement général susceptible d’être évalué.

Cela est important pour le codage agentique car les agents peuvent créer rapidement de grandes différences. Une bonne suite d’instantanés rend ces différences visibles sous une forme que les humains peuvent consulter. Cela transforme « quelque chose a changé quelque part dans le compilateur » en « cette sortie de rendu a changé exactement dans ce cas ».

C’est une bien meilleure surface de critique.

## Le déterminisme est le contrat

Les flux de travail agents nécessitent des outils déterministes.

Si les tests sont instables, l’agent ne peut pas dire si son patch a aidé. Si l’ordre de sortie change entre les exécutions, les instantanés deviennent du bruit. Si les diagnostics dépendent de l’état ambiant de la machine, le CI devient une loterie.

Donc Vize se soucie des détails ennuyeux :

- Ordonnancement stable des sorties
- ID diagnostique stable
- Plages sources stables
- Forme de code stable générée
- Installation stable de luminaires
- Annuaires isolés à l’article

Le déterminisme n’est pas réservé uniquement à l’IC. C’est ce qui permet aux humains et aux agents de partager la même boucle de rétroaction.

## Les installations réelles maintiennent le système honnête

Les tests unitaires sont nécessaires, mais les outils Vue vivent dans les projets réels.

Les projets réels ont :

- Graphes d’importation inhabituels
- Dispositions du gestionnaire de packages
- Fichiers générés
- Conventions macro
- Préprocesseurs de style
- Arbres composants énormes
- Anciens motifs à côté de nouveaux motifs

C’est pourquoi Vize continue de tester sur des fixatures et des instantanés réels. L’objectif n’est pas de déclarer une préparation à la production trop tôt. Le but est de trouver chaque arête qui n’apparaît qu’en dehors d’une application d’échantillons parfaite.

Ce type de vérification exhaustive est lent à se construire, mais c’est le chemin de l’expérimentation vers un véritable outil.

## Les tests sont une conversation avec la communauté

Les retours de la communauté ne sont pas seulement des commentaires de suivi.

Il est également :

- un vrai projet qui ne parvient pas à compiler
- un diagnostic qui indique une mauvaise plage de temps
- un faux positif qui bloque l’adoption
- Un précipice de performance dans un dépôt que personne n’avait prédit
- un schéma de production que la chaîne d’outils ne comprenait pas

Chacun de ces rapports devrait devenir un point de référence, un test de régression ou un point de référence.

C’est ainsi que le retour d’information devient mémoire. C’est ainsi qu’un outil expérimental devient plus sérieux avec le temps.

## Les agents ont besoin de boucles plus petites et meilleures

La pire configuration de test pour les agents est une grosse commande lente qui échoue à la fin avec un message flou.

La meilleure configuration donne un retour en couches :

- Tests unitaires rapides pour les invariants locaux
- Tests instantanés pour la révision des sorties
- Tests de fixture pour le comportement du cadre
- Tests d’intégration ciblés pour les limites des outils
- Matrices CI pour plateformes et constructions de production

Les agents peuvent utiliser cette échelle. Les humains aussi.

C’est une des raisons pour lesquelles Vize continue d’investir dans des outils de test et la consolidation de scripts. Un bon projet doit rendre le bon chèque facile à exécuter, facile à comprendre et facile à étendre lorsque le risque augmente.

## La confiance se gagne à plusieurs reprises

Aucune chaîne d’outils ne devient fiable parce que son README indique « rapide » ou « correct ».

La confiance se gagne à chaque fois :

- Un diagnostic est précis
- Une réparation ne détériore pas le code à proximité
- Un changement instantané est explicable
- Un projet réel continue de passer
- Le CI détecte quelque chose avant la libération
- Un agent peut itérer sans perdre le thread

C’est pourquoi les tests ne sont pas une quête annexe pour Vize.

Cela fait partie du produit.

À l’ère de l’IA, les meilleurs outils ne seront pas ceux qui génèrent le plus de code. Ce sont eux qui pourront générer, valider, expliquer et rejeter du code dans des boucles serrées et déterministes.

Les tests sont là où ces boucles deviennent réelles.
