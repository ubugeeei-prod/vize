---
title: Caisses
---

<!-- Generated translation; source: architecture/crates.md -->

# Référence à la caisse

> **⚠️ Travaux en cours :** Vize est en développement actif. Voir le canonique
> [Rust crate support tiers](../stability.md#rust-crate-support-tiers) avant de dépendre d’un public
> API.

L’espace de travail Rust de Vize est organisé autour de 20 caisses principales. Chaque caisse possède une voie réutilisable afin que
analyse syntaxique, analyse sémantique, génération de code, linting, formatage, vérification de types et outils de
éditeur puissent partager le même modèle syntaxique.

## Fondation

| Caisse            | Rôle                                                                                                 |
| ----------------- | ---------------------------------------------------------------------------------------------------- |
| `vize_carton`     | Alloquateur partagé, chaînes, collections de hachages, flags, profiler, i18n et utilitaires DOM/tags |
| `vize_relief`     | AST du modèle Vue partagé, erreurs du compilateur et options du compilateur                          |
| `vize_armature`   | Tokenizer et parseur de modèles Vue                                                                  |
| `vize_croquis`    | Analyse sémantique, suivi de la portée, métadonnées de liaison, réactivité et assistants TS virtuels |
| `vize_croquis_cf` | Analyse sémantique inter-fichiers et diagnostics à l’échelle du projet                               |

## Compilation

| Caisse               | Rôle                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------- |
| `vize_atelier_core`  | Voie de transformation partagée et infrastructure de génération de code                 |
| `vize_atelier_dom`   | Compilation de modèles orientée VDOM                                                    |
| `vize_atelier_vapor` | Compilation de modèles en mode vapeur                                                   |
| `vize_atelier_ssr`   | Compilation de modèles de rendu côté serveur                                            |
| `vize_atelier_sfc`   | `.vue` analyse syntaxique ainsi que l’orchestration de scripts, de modèles et de styles |
| `vize_atelier_jsx`   | Analyse partagée, réduction et intégration du compilateur JSX/TSX                       |

## Outils de développement

| Caisse         | Rôle                                                                                                 |
| -------------- | ---------------------------------------------------------------------------------------------------- |
| `vize_patina`  | Mise en forme du linter et diagnostic du SFC Vue                                                     |
| `vize_glyph`   | Formateur SFC Vue                                                                                    |
| `vize_canon`   | Vérification de type consciente de Vue et génération virtuelle de TypeScript                         |
| `vize_maestro` | Implémentation du protocole Language Server                                                          |
| `vize_musea`   | Analyse syntaxique de l’art Musea, documentation, génération de palettes, autogénération et cœur VRT |
| `vize_curator` | Charges utiles d’inspecteur local, métadonnées graphique/diff et rapports de profil                  |
| `vize_fresco`  | Primitives d’interface utilisateur terminale utilisées par les expériences orientées TUI             |

## Couches de distribution

| Caisse         | Rôle                                                             |
| -------------- | ---------------------------------------------------------------- |
| `vize_vitrine` | Liaisons NAPI et WASM partagées pour les consommateurs JS        |
| `vize`         | CLI Rust-native plus réexportations en caisse pour les documents |

## Notes

- `vize_musea` 'est le noyau Rust pour les outils d’art de Musea. L’interface de la galerie et le flux de travail dev-server sont
  fourni par `@vizejs/vite-plugin-musea`.
- `vize_curator` n’est pas publié. Elle possède des artefacts de développeurs locaux tels que des charges utiles d’inspecteur,
  rapports d’agent, métadonnées de graphes croisés et rendu de rapports de profil CLI. Le profileur de
  de bas niveau reste en `vize_carton` car les caisses partagées utilisent leurs propres chemins chauds.
- `vize_vitrine` est le pont entre Rust et JS. Des paquets tels que `@vizejs/native` et
  `@vizejs/wasm` publie ses reliures.
  - `vize` est la caisse complète de Rust CLI dans l’espace de travail. Pour la version alpha v1, son canal binaire public est
    GitHub Releases ou Nix, tandis que le paquet npm `vize` est le point d’entrée du package script pris en charge.

## Cartographie des paquets

| Package / Commande          | Caisse principale de rouille                                                             |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `vize build`                | `vize`, `vize_atelier_sfc`, `vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr` |
| `vize fmt`                  | `vize`, `vize_glyph`                                                                     |
| `vize lint`                 | `vize`, `vize_patina`                                                                    |
| `vize check`                | `vize`, `vize_canon`                                                                     |
| `vize inspector`            | `vize`, `vize_curator`                                                                   |
| `vize lsp`                  | `vize`, `vize_maestro`                                                                   |
| `@vizejs/vite-plugin`       | `vize_vitrine`, `vize_atelier_sfc`                                                       |
| `@vizejs/native`            | `vize_vitrine`                                                                           |
| `@vizejs/wasm`              | `vize_vitrine`                                                                           |
| `@vizejs/vite-plugin-musea` | `vize_musea`, `vize_vitrine`                                                             |
| `@vizejs/musea-mcp-server`  | `vize_musea`, `vize_vitrine`                                                             |
| `oxlint-plugin-vize`        | `vize_patina`, `vize_vitrine`                                                            |
