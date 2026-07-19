---
layout: entry
title: Vize
description: Chaîne à outils haute performance Vue.js en rouille. Compiler, liner, formater, vérifier le type et explorer les composants Vue.
hero:
  name: Vize
  text: Chaîne d’outils Vue.js haute performance dans la rouille
  tagline: /viːz/ — Un outil avisé qui voit clair dans votre code. Compiler, lintuer, formater, vérifier les tapes et explorer les composants Vue — le tout alimenté par Rust. ⚠️ Pas encore prêts pour la production.
  image:
    src: logo.svg
    alt: Vize Logo
  actions:
    - theme: brand
      text: Commencez
      link: fr/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: Aire de jeux
      link: https://vizejs.dev/play
features:
  - title: Vite Plugin
    details: "Partez de l’intégration recommandée pour les applications Vue : compilation SFC native dans Vite avec une configuration partagée de Vize."
    link: fr/guide/vite-plugin.md
  - title: Pipeline d’analyse statique
    details: L’analyse syntatique, l’analyse sémantique, les règles de lint, le TypeScript virtuel, les vérifications entre fichiers et les diagnostics de l’éditeur partagent les mêmes couches d’analyse native Rust.
    link: fr/guide/static-analysis.md
  - title: Documentation des règles
    details: Parcourez Concrete Vue, HTML, SSR, Vapor, Musea, typ-aware, et croisez les diagnostics avec des bons et mauvais exemples.
    link: fr/rules/index.md
  - title: Configuration partagée
    details: Configurez les options du compilateur, le scan Vite, les préréglages de lint, la vérification de type, la mise en forme, les fonctionnalités LSP et Musea de `vize.config.*`.
    link: fr/guide/configuration.md
  - title: Vérification de type native
    details: |
      `vize:check` scripts de paquet s’exécutent à travers des sessions de projet `vize_canon` et Corsa, soutenues par `corsa-bind`, maintenant les diagnostics conscients de Vue sur un chemin natif.
    link: fr/guide/static-analysis.md
  - title: Scripts de paquets et référence CLI
    details: Utilisez le package npm des scripts de projet pour les flux de travail des applications, avec la ligne de commande Rust documentée pour le LSP, le profilage et l’utilisation directe du binaire.
    link: fr/guide/cli.md
  - title: Inspecteur de compilateur
    details: Inspectez la sortie Vue, la sortie Vize, Virtual TS, VIR, et les graphiques croisés, puis partagez des repros ou rapports d’agents liés en permalink.
    link: fr/guide/compiler-inspector.md
  - title: Oxlint Plugin
    details: Lance les diagnostics Vue de Vize dans Oxlint et combine-les avec les règles JS et TS d’OXC en une seule passe.
    link: fr/guide/oxlint.md
  - title: Intégrations expérimentales de bundlers
    details: il existe un rollup, webpack, esbuild et un chemin dédié Rspack, mais Vite reste l’intégration recommandée et la plus stable.
    link: fr/guide/unplugin.md
  - title: 8,3 fois plus rapide
    details: Compilation multithread de 15 000 fichiers SFC (36,9 Mo) en moins de 500 ms. Allocation d’arène, parallélisme rayonne, zéro GC.
    link: fr/architecture/performance.md
  - title: Galerie composante
    details: Musea — fichiers d’art, documentation, génération de palettes, outils a11y et VRT, avec le flux de travail de la galerie fourni par @vizejs/vite-plugin-musea.
    link: fr/guide/musea.md
  - title: Reliures WASM
    details: Exécutez le compilateur Vue directement dans le navigateur avec WebAssembly. Des terrains de jeux énergétiques, des docs et des outils éducatifs.
    link: fr/guide/wasm.md
  - title: Intégration de l’IA
    details: Serveur MCP permettant aux assistants IA de comprendre et de travailler avec vos composants Vue via Musea.
    link: fr/integrations/mcp.md
  - title: Mode vapeur
    details: Prise en charge de première classe pour le mode Vapeur Vue 3.6 — compilation réactive fine sans le DOM virtuel.
    link: fr/architecture/overview.md
  - title: Philosophie
    details: Architecture inspirée de l’art, écosystème d’oxydation (OXC, oxlint, corsa-bind), et une vision unifiée de la chaîne d’outils.
    link: fr/philosophy.md
  - title: Blog
    details: Des notes de version pour les modifications expédiées, ainsi que des notes irrégulières pour les mises à jour de conception, les devlogs et la réflexion projet.
    link: fr/blog/index.md
---

<!-- Generated translation; source: index.md -->

## Direction actuelle

L’un des plus grands changements récents chez Vize est la vérification des types natifs. La commande `vize check` utilisée par les scripts de paquets npm
et le pipeline de vérification de type orienté éditeur sont transférés sur `vize_canon` plus
[`corsa-bind`](https://github.com/ubugeeei/corsa-bind), ce qui permet à Vize de conserver plus longtemps les fichiers virtuels
Vue et les diagnostics de projets TypeScript sur un chemin natif.

Cela compte plus que la vitesse brute. Cela offre à Vize une boucle plus resserrée entre l’analyse des modèles, le diagnostic, la navigation et les futures fonctionnalités de l’éditeur, tout en réduisant la charge de travail à remettre en place via un processus de compilateur hébergé en JavaScript. L’histoire de la fidélité est encore en train de rattraper, mais c’est clairement la direction que prend la chaîne d’outils.

La même direction s’applique à la linting et à la musea. L’analyse statique commence avec l’analyseur syntaxique et Croquis
modèle sémantique, puis alimente les règles de lint Patina, le TypeScript virtuel Canon, les décisions du compilateur, les diagnostics de
éditeur, ainsi que les métadonnées de la galerie de composants. Le flux de travail pratique est documenté dans
[Static Analysis](./guide/static-analysis.md), avec les détails de configuration dans
[Configuration](./guide/configuration.md). La règle concrète et le catalogue de diagnostic sont en
[Rules](./rules/index.md).

## Auteur

![ubugeeei](https://github.com/ubugeeei.png)

- \*[ubugeeei](https://github.com/ubugeeei)\*\* est ingénieur logiciel basé à Tokyo, travaillant dans les domaines Vue, Rust, design et outils de langage.

Il fait partie de l’équipe centrale [Vue.js Core Team](https://vuejs.org/about/team.html) [Vue.js Japan User Group](https://github.com/vuejs-jp) , contributeur principal [Vite+](https://github.com/voidzero-dev/vite-plus) et ingénieur en chef chez [mates-dev](https://github.com/mates-dev).

Il est aussi le créateur de [chibivue](https://github.com/chibivue-land/chibivue), [Vize](https://github.com/ubugeeei-prod/vize)et [Ox Content](https://github.com/ubugeeei/ox-content).

- GitHub : [github.com/ubugeeei](https://github.com/ubugeeei)
- X (Twitter) : [@ubugeeei](https://x.com/ubugeeei)
- Blog : [wtrclred.io](https://wtrclred.io)
- chibivue.land : [chibivue.land](https://chibivue.land)

## Parrain

Vize est un projet libre et open source sous licence MIT. Développer et maintenir une chaîne d’outils complète — compilateur, linter, formateur, vérificateur de type, LSP, galerie de composants et liaisons WASM — est un effort important qui nécessite une concentration et un engagement soutenus.

Si Vize vous fait gagner du temps, améliore votre expérience de développement, ou si vous croyez en la vision d’une chaîne d’outils Vue.js haute performance, veuillez envisager de parrainer le projet :

- L’infrastructure CI/CD runner est sponsorisée par [Blacksmith](https://www.blacksmith.sh/).
- [GitHub Sponsors](https://github.com/sponsors/ubugeeei)

Votre soutien aide à financer le développement continu, les coûts d’infrastructure, et garantit que Vize reste gratuit pour tous. Chaque contribution — quelle que soit la taille — fait une réelle différence.
