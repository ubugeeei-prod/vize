---
title: Philosophie
---

<!-- Generated translation; source: philosophy.md -->

# Philosophie

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les principes de conception ci-dessous décrivent la vision et l’orientation du projet.

Vize est plus qu’un simple compilateur — c’est une déclaration de conception sur la façon dont Vue.js outils.

## Pourquoi Vize existe

L’écosystème JavaScript s’est longtemps appuyé sur des outils basés sur JavaScript pour compiler, liner, formater et vérifier le code JavaScript. Cela crée un goulot d’étranglement fondamental : les outils qui traitent votre code sont soumis aux mêmes limitations d’exécution que le code qu’ils traitent — pauses de collecte des ordures, exécution en un seul thread et overhead de dispatch dynamique.

Vize adopte une approche différente. En réécrivant toute la chaîne d’outils Vue.js dans Rust, nous éliminons ces contraintes au niveau de l’architecture. Le résultat n’est pas une amélioration progressive — c’est un changement catégorique dans ce qui est possible.

## Principes de conception

### 1. Chaîne d’outils unifiée

Le développement traditionnel Vue.js nécessite d’assembler une constellation d’outils distincts : un compilateur (`@vue/compiler-sfc`), un linter (eslint + eslint-plugin-vue), un formateur (plus joli), un vérificateur de type (vue-tsc) et un explorateur de composants (Storybook). Chaque outil possède son propre analyseur, sa propre représentation AST et son propre format de configuration.

Vize unifie tous ces éléments en un seul binaire. Un seul parser. Un AST. Une surface de configuration. Cela élimine les passes d’analyse redondant, réduit la complexité de configuration et garantit que tous les outils partagent une compréhension cohérente de votre code.

```
@vue/compiler-sfc  +  eslint-plugin-vue  +  prettier  +  vue-tsc  +  Storybook
                              ↓
                            vize
```

### 2. Performance en tant que fonctionnalité

La vitesse n’est pas un atout agréable — c’est un prérequis pour une expérience développeur. Quand la compilation prend quelques secondes, les développeurs perdent du flow. Quand le linting prend quelques minutes, les développeurs le désactivent. Lorsque la vérification de type prend trop de temps, les développeurs la sautent.

Vize est conçu pour que chaque outil fonctionne suffisamment vite pour être utilisé de manière interactive :

- **Compilation** : 15 000 fichiers SFC en 498 ms (multi-thread)
- **Mise en forme** : quasi-instantané, même sur de grandes bases de code
- **Linting** : retour en temps réel via le LSP
- **Vérification de type** : analyse incrémentale sans overhead V8

Cela est réalisé grâce aux abstractions gratuites de Rust, à l’allocation d’arène et au multithreading natif avec Rayon.

### 3. Compatibilité en direct

Vize ne vous demande pas de réécrire votre code ni de modifier votre flux de travail. Le plugin Vite est un remplacement direct de `@vitejs/plugin-vue`. Vos composants existants de Vue, `<script setup>`, styles avec lunette et HMR fonctionnent tous sans modification.

Ce principe s’étend à l’écosystème plus large. Le plugin Vite de Vize est compatible avec Nuxt, et le LSP s’intègre à VS Code via des protocoles standards. Adopter Vize devrait donner l’impression d’améliorer son moteur, pas de reconstruire sa voiture.

### 4. L’art comme architecture

Chaque caisse Vize porte le nom d’un concept issu des arts visuels — peinture, sculpture et conservation muséale. Ce n’est pas une pure fantaisie. La convention de nommage encode une philosophie : **le code est un médium créatif**, et les outils qui le façonnent doivent refléter l’artisanat impliqué.

| Caisse       | Origine artistique                | Rôle                                        |
| ------------ | --------------------------------- | ------------------------------------------- |
| **Carton**   | Dossier de portfolio d’artiste    | Utilitaires partagés — la boîte à outils    |
| **Relief**   | Projection sculpturale de surface | AST — la surface structurée du code         |
| **Armature** | Squelette soutenant une sculpture | Analyseur syntaxique — le cadre structurel  |
| **Croquis**  | Esquisse gestuel rapide           | Analyse sémantique — capture de l’essence   |
| **Atelier**  | Atelier d’artiste                 | Compilateur — où la transformation a lieu   |
| **Vitrine**  | Vitrine en verre                  | Reliures — exposant l’œuvre                 |
| **Canon**    | Standard des proportions idéales  | Vérificateur de type — garantir la justesse |
| **Patine**   | Qualité de surface vieillie       | Linter — polissage de la surface            |
| **Glyphe**   | Symbole ou forme de lettre gravée | Formateur — façonnage du texte              |
| **Maestro**  | Chef d’orchestre principal        | LSP — orchestrer l’expérience               |
| **Musea**    | Pluriel de musée                  | Galerie composante — exposition de l’œuvre  |
| **Fresque**  | Technique de peinture murale      | Cadre TUI — peindre le terminal             |

Ce système de dénomination a un but pratique : il rend la hiérarchie des caisses intuitive. Quand vous voyez `vize_atelier_dom`, vous comprenez immédiatement qu’il s’agit d’un _atelier_ qui produit _des sorties VDOM_. Quand vous voyez `vize_patina`, vous savez que cela _polissent_ votre code.

#### L’analogie de la sculpture

L’analogie la plus profonde se situe entre la compilation logicielle et la sculpture. Considérez comment travaille un sculpteur :

1. **Armature** — Le sculpteur commence par construire une armature : un squelette en fil de fer qui définit la structure de base. Dans Vize, l’analyseur (`vize_armature`) construit le cadre structurel (AST) à partir du texte source brut.

2. **Relief** — Le sculpteur construit la surface sur l’armadura, créant un _relief_ — une surface structurée qui s’avance depuis un plan plat. Dans Vize, l’AST (`vize_relief`) donne une forme structurée et tridimensionnelle à ce qui était à l’origine un texte plat.

3. **Croquis** — Avant de s’engager dans une sculpture finale, l’artiste réalise des croquis rapides (_croquis_) pour comprendre le caractère essentiel du sujet. Dans Vize, l’analyse sémantique (`vize_croquis`) est un passage rapide qui capture le sens du code — quelles variables sont liées, quelles expressions sont valides — sans s’engager dans une cible de compilation.

4. **Atelier** — Le sculpteur se déplace vers _l’atelier_ (atelier) pour créer la pièce finale. Plusieurs ateliers peuvent produire différentes interprétations du même sujet. Dans Vize, les backends de compilation (`vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr`) sont différents ateliers qui produisent différentes versions (VDOM, Vapor, SSR) du même AST analysé.

5. **Vitrine** — L’œuvre finie est placée dans une _vitrine_ (vitrine en verre) afin que d’autres puissent l’observer. Dans Vize, les liaisons (`vize_vitrine`) sont une couche transparente qui permet aux consommateurs JavaScript d’accéder à la sortie compilée.

6. **Musea** — Enfin, les œuvres sont exposées dans un _musée_ pour leur appréciation et leur étude. Dans Vize, la galerie de composants (`vize_musea`) est l’endroit où les composants sont exposés, explorés et documentés.

#### L’analogie des métiers de qualité

Les caisses restantes suivent une analogie de savoir-faire :

- **Canon** (vérificateur de type) — En sculpture classique, le _canon_ était une norme d’une dimension humaine idéale. Polykleitos a écrit le _Kanon_ définissant les rapports mathématiques pour la figure parfaite. Dans Vize, le vérificateur de caractères impose les « proportions idéales » de votre code — les types doivent être corrects, les accessoires doivent correspondre, les émissions doivent être conformes.

- **Patine** (linter) — Une _patine_ est la finition de surface qui se développe sur des matériaux vieillissants, indiquant la qualité et le soin. Une sculpture en bronze à la riche patine a été bien entretenue. Dans Vize, le linter examine la surface de votre code, identifiant les problèmes qui affectent sa qualité.

- **Glyphe** (formateur) — Un _glyphe_ est un symbole ou une forme de lettre gravée — pensez aux formes de lettres précises et cohérentes dans une police. Chaque glyphe a des proportions et des espacements exacts. Dans Vize, le formateur garantit que votre code a des proportions cohérentes et précises.

- **Maestro** (LSP) — Un _maestro_ est le chef d’orchestre qui orchestre un ensemble en une performance unifiée. Dans Vize, le serveur LSP orchestre toutes les fonctionnalités du langage (complétion, diagnostic, mise en forme, navigation) dans une expérience d’éditeur unifiée.

- **Fresque** (TUI) — Une _fresque_ est une technique de peinture où un pigment est appliqué sur un plâtre humide, devenant ainsi une partie intégrante du mur lui-même. Dans Vize, le cadre TUI « peint » directement sur la surface du terminal.

### 5. Pensée d’abord en mode vapeur

Vue 3.6 introduit le mode Vapor — une stratégie de compilation qui génère du code réactif à grains fins sans le DOM virtuel. Vize a été conçu avec le mode Vapor comme cible de compilation de première classe dès le premier jour.

Alors que `@vue/compiler-sfc` ajouté progressivement le support Vapor, le `vize_atelier_vapor` de Vize a été construit parallèlement à `vize_atelier_dom` dès le début. Cela signifie que l’infrastructure de compilation partagée (`vize_atelier_core`) est conçue pour servir les deux modes de sortie de manière égale.

### 6. Souveraineté des développeurs

Vize est une chaîne **d’outils indépendante** . Il n’est pas contrôlé par l’équipe centrale Vue.js, et ne revendique pas d’être la manière « officielle » de construire des applications Vue. C’est intentionnel.

En restant indépendante, Vize peut :

- Expérimentez des stratégies de compilation sans la contrainte de la rétrocompatibilité
- Avancer plus vite qu’un projet officiel soumis à des processus de gouvernance
- Servir de terrain d’essai pour des idées qui pourraient éventuellement influencer la chaîne d’outils officielle
- Offrir une alternative aux développeurs qui souhaitent des performances maximales

En même temps, Vize suit de près les spécifications officielles Vue.js. L’objectif est la compatibilité, pas la fragmentation.

### 7. Debout sur les épaules de l’oxydation

Vize n’existe pas isolément. Elle s’inscrit dans un mouvement plus large visant à réécrire les outils JavaScript dans les langages système — ce que la communauté appelle « oxydation ». Vize adopte et s’intègre à cet écosystème :

- **OXC** — Vize utilise le [Oxidation Compiler](https://oxc.rs/) (oxc) pour l’analyse JavaScript et TypeScript. OXC fournit l’analyse AST JS/TS haute performance qui alimente `vize_croquis` (analyse sémantique) et `vize_atelier_core` (génération de code). Plutôt que de réimplémenter un analyseur JS, Vize délègue à l’implémentation éprouvée d’OXC.
- **oxlint** — Vize est conçu en pensant à [oxlint](https://oxc.rs/docs/guide/usage/linter) . Bien que `vize_patina` gère le linting spécifique à Vue, l’histoire plus large du linting JavaScript est mieux servie par le moteur de règles natif Rust d’oxlint. Les deux outils sont complémentaires, pas concurrents.
- **Corsa** — La couche native d’exécution TypeScript de Vize, construite autour de [`corsa-bind`](https://github.com/ubugeeei/corsa-bind), représente la direction que Vize prend pour la vérification des types JavaScript/TypeScript sans tout faire passer par un compilateur hébergé en JavaScript. `vize_canon` utilise cette pile pour des diagnostics natifs tout en continuant à fournir une analyse de types de modèles spécifique à Vue.
- **LightningCSS** — Vize utilise [LightningCSS](https://lightningcss.dev/) pour l’analyse et la transformation CSS au sein de `vize_atelier_sfc`, en tirant parti de son traitement CSS natif Rust pour les styles à portée optique.

Il reste encore de nombreux défis non résolus dans ce domaine — interopérabilité AST entre outils, analyse incrémentale au-delà des frontières linguistiques, et cohérence de l’intégration de l’éditeur. Vize vise à être un terrain d’essai pour des solutions à ces problèmes au sein de l’écosystème Vue.js, contribuant ainsi au mouvement plus large de l’oxydation.

### 8. Collaboration avec Vite+ et OXC

[Vite+](https://viteplus.dev/) et [OXC](https://oxc.rs) sont des chaînes d’outils **indépendantes du cadre** — elles fournissent des capacités de regroupement, d’analyse syntaxique, de linting et de formatage JS/TS/CSS à usage général qui fonctionnent sur n’importe quel framework. Vize est **spécifique à Vue** et est conçu pour **s’intégrer à** ces outils d’écosystème plutôt que pour les concurrencer.

Vize dépend directement d’OXC pour l’analyse JavaScript/TypeScript et de LightningCSS pour le traitement CSS au sein des SFC Vue. Le linter (patine) et le formateur (glyphe) de Vize traitent des préoccupations spécifiques à Vue (directives de modèles, structure SFC, conventions de composants) qui ne relèvent pas du cadre des outils indépendants du cadre. Une intégration plus approfondie avec OXC est prévue — par exemple, déléguer `<script>` linting/formatage de blocs à OXC tandis que Vize gère les couches de coordination `<template>` et SFC spécifiques à Vue. Le plugin Vite de Vize (`@vizejs/vite-plugin`) est construit sur Vite et conçu pour remplacer directement `@vitejs/plugin-vue`, intégrant pleinement l’écosystème Vite.

En tant qu’auteur de Vize, je ([@ubugeeei](https://github.com/ubugeeei)) veux être clair : **je n’ai aucune intention adversaire envers aucun de ces projets.** Je suis totalement ouverte à la collaboration et je crois que les meilleurs résultats viennent d’outils qui se complètent. Si des changements sont nécessaires de part et d’autre pour permettre une meilleure intégration, je suis prêt à travailler ensemble pour y parvenir.

## Le nom

- _Vize\*\* (_/viːz/\*) est dérivé de trois mots :

* **Vizir** — un conseiller ou un conseiller sage
* **Visière** — quelque chose qui vous aide à voir clairement
* **Conseiller** — un guide qui vous aide à prendre de meilleures décisions

Ensemble, ils décrivent un outil qui _voit à travers votre code_ et _vous conseille judicieusement_. La prononciation rime avec « breeze » — rapide, sans effort et rafraîchissante.
