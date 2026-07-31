---
title: Bien démarrer
---

<!-- Generated translation; source: getting-started.md -->

# Bien démarrer

> **⚠️ En cours de développement :** Vize évolue activement et n’est pas encore prêt pour la
> production. Les API et les frontières entre paquets peuvent changer sans préavis.

Vize (_/viːz/_) est une chaîne d’outils Vue.js native en Rust. Elle réunit la compilation, le lint,
le formatage, la vérification de types, les diagnostics dans l’éditeur et l’exploration de composants
dans un même workspace, tout en proposant chaque fonction via des paquets et commandes spécialisés.

| Besoin                                                                                | Point d’entrée recommandé   |
| ------------------------------------------------------------------------------------- | --------------------------- |
| Compiler des SFC Vue avec Vite                                                        | `@vizejs/vite-plugin`       |
| Compiler des SFC Vue avec Nuxt                                                        | `@vizejs/nuxt`              |
| Lancer le lint, le formatage et la vérification de types depuis les scripts du projet | `vize`                      |
| Combiner les diagnostics Vize avec Oxlint                                             | `oxlint-plugin-vize`        |
| Explorer et tester les composants                                                     | `@vizejs/vite-plugin-musea` |
| Évaluer les fonctions d’édition                                                       | VS Code, Zed ou `vize lsp`  |

## Configurer un projet existant

Lancez l’initialisation interactive à la racine du projet :

```bash
vpx vize init
```

`vpx` est fourni avec [Vite+](https://viteplus.dev/guide/install). Installez d’abord Vite+ si la
commande n’est pas disponible dans votre shell.

Avant toute écriture, `vize init` détecte Vite, Vite+ ou Nuxt, le gestionnaire de paquets,
TypeScript, la commande de lint active et la configuration Vize existante. Vous choisissez les
éléments à configurer :

- le plugin Vite ou le module Nuxt
- le plugin Oxlint, dans le fichier réellement lu par la commande de lint active
- les scripts de projet `vize fmt` et `vize check`
- les réglages partagés `vize.config.*`
- une recommandation d’extension VS Code

Prévisualisez toutes les modifications de fichiers et de dépendances sans rien écrire :

```bash
vpx vize init --dry-run
```

Dans la CI ou tout autre environnement non interactif, sélectionnez explicitement les fonctions :

```bash
vpx vize init --yes --lint --bundler --fmt --typecheck --editor
```

Consultez [Project Setup (en anglais)](../guide/init.md) pour les règles de détection, toutes les
options, les garanties d’idempotence et les cas où l’initialiseur refuse volontairement de modifier
un fichier.

## Choisir une configuration manuelle

Préférez la configuration manuelle pour préserver une configuration existante ou adopter une seule
partie de Vize à la fois :

- [Plugin Vite](./guide/vite-plugin.md) — compilation native des SFC Vue dans Vite
- [Intégration Nuxt](./integrations/nuxt.md) — voie prise en charge dans le pipeline Vite de Nuxt
- [Scripts de paquet et CLI](./guide/cli.md) — `vize build`, `fmt`, `lint`, `check`, `ready` et la CLI
  Rust complète

Vite est l’intégration de bundler recommandée. Les paquets unplugin et Rspack restent expérimentaux ;
leur périmètre actuel est décrit dans [Autres bundlers](./guide/unplugin.md).

## Consulter les guides spécialisés

Cette page sert volontairement de point d’orientation. Pour les détails de configuration et
d’intégration, les guides spécialisés constituent la source de référence :

- [Configuration](./guide/configuration.md) — `vize.config.*`, options du compilateur, vérification
  de types et réglages Musea
- [Analyse statique](./guide/static-analysis.md) — modèle de lint et de vérification de types
- [Documentation des règles](./rules/index.md) — diagnostics concrets et exemples
- [Plugin Oxlint](./guide/oxlint.md) — préréglages, options et fichier de configuration réellement lu
  par chaque commande
- [VS Code et autres éditeurs](./integrations/vscode.md) — profil d’édition optionnel et configuration LSP
- [JSX et TSX](./guide/jsx.md) — composants Vue écrits hors des SFC `.vue`
- [Musea](./guide/musea.md) — exemples, documentation, jetons, a11y et VRT des composants

Tant que l’intégration de Vize aux éditeurs reste expérimentale, continuez à utiliser l’outil
officiel [`vuejs/language-tools`](https://github.com/vuejs/language-tools) au quotidien.
