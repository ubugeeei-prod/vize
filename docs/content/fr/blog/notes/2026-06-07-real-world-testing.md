---
title: Tests dans le monde réel
description: Vize entre dans la phase de Real World Testing — les projets réels sont désormais la suite de tests, avec une feuille de route claire vers la version 1.0.0.
---

<!-- Generated translation; source: blog/notes/2026-06-07-real-world-testing.md -->

# Tests dans le monde réel

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publié le</span>
<span class="blog-meta-value">07-06-2026</span>
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

<video class="blog-post-video" src="/blog/vize-real-world-testing.mp4" controls muted playsinline loop preload="metadata" aria-label="Real World Testing PV"></video>

Vize entre dans une nouvelle phase.

Jusqu’à présent, le développement s’est concentré sur la mise en œuvre de fonctionnalités, la construction d’infrastructures, la
et la validation des comportements via des suites de tests dédiées et des exemples synthétiques.

L’étape suivante est différente.

Nous recherchons désormais **activement des projets concrets** pour tester Vize.

## L’objectif

L’objectif de cette phase est de découvrir les lacunes de compatibilité, les lacunes de spécification,
les goulots d’étranglement de performance et les cas particuliers qui n’apparaissent que dans les bases de code de production.

Si vous maintenez une application, une bibliothèque, un framework ou un outil Vue, nous serions ravis d’entendre
sur votre expérience de l’utilisation avec Vize.

Chaque rapport de correction, reproduction, résultat de benchmark et recherche de compatibilité aide à rapprocher le projet
sa première version stable.

## Toujours expérimental — D’abord la correction

Vize devrait toujours être considéré comme expérimental. Des changements incontrôlables peuvent survenir, les correctifs ne sont
attendus, et le comportement peut différer de celui de Vue dans certains cas.

L’objectif de cette phase n’est pas le développement de fonctionnalités. L’objectif est la justesse.
Les applications réelles sont désormais la suite de tests. Si vous rencontrez quelque chose qui nécessite une correction, merci de
le signaler — chaque rapport aide à améliorer le compilateur, la spécification du langage et
l’écosystème global.

## Comment aider

Nous attendons de nombreuses demandes de réparation et de répertoriations. Nous recrutons également activement des projets Vue assez
importants pour les utiliser comme bancs d’essai — plus la base de code est grande et réelle, plus le signal est
précieux. Si vous maintenez (ou connaissez) une application, une bibliothèque, un framework
ou un outil Vue important, veuillez ouvrir une demande de correction ou contacter pour que nous puissions exécuter Vize contre cette application. Les rapports de
correctifs, les reproductions et les résultats de benchmark sont tous les bienvenus.

Consultez le guide [Testing & Feedback](../../guide/testing.md) pour savoir comment inspecter les résultats dans le terrain de jeu
, lisez les cas de test existants, le profil avec `vize check --profile`, et proposez un projet
en tant que banc d’essai E2E / VRT.

## Feuille de route vers la version 1.0.0

La phase actuelle est **le Real World Testing**.

Une fois que Vize aura réussi cette phase, le projet passera à travers :

- v1.0.0-alpha
- v1.0.0-beta
- v1.0.0-rc
- v1.0.0

Les phases alpha, bêta et candidate à la version se concentreront sur la stabilisation, la compatibilité
de l’écosystème, les améliorations de performance et les garanties de maintenance à long terme.

L’objectif n’est pas de se précipiter vers la 1.0. Le but est de le mériter.

Si vous souhaitez contribuer à façonner l’avenir de Vize, c’est le meilleur moment pour vous impliquer
.
