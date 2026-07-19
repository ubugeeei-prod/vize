---
title: Stabilité
description: Les niveaux de support alpha de Vize v1, promesses de compatibilité et surfaces expérimentales.
---

<!-- Generated translation; source: stability.md -->

# Stabilité

Vize se dirige vers une version alpha v1. Le contrat alpha est intentionnellement plus étroit qu’un contrat
v1 stable : il nomme les surfaces qui devraient être utilisables par les premiers adopteurs, tout en laissant la place pour
modifier rapidement les internes et les intégrations expérimentales. Le projet complet n’est pas encore une chaîne d’outils entièrement
prête pour la production ; Les décisions de publication doivent utiliser les
[production-readiness checklist](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md).
fenêtres de dépréciation, règles SemVer et support des lignes de sortie sont précisés dans le
[support policy](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md).

## Contrat de gestion de versions

Avant la version 1 stable, toute pré-version peut inclure des changements cassants. Vize considère toujours les changements interrompus comme
matériel de release-note, surtout lorsqu’ils affectent les points d’entrée de paquets, les drapeaux CLI, les champs de configuration
les codes de diagnostic ou les sorties générées.

La ligne alpha v1 utilise ces règles :

| Surface                                            | Attente alpha                                                                                                          |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Noms de paquets publiés                            | Elles doivent rester disponibles ou être expédiées avec des notes de migration                                         |
| Commandes et drapeaux CLI documentés               | Il faut éviter les changements de comportement silencieux                                                              |
| Champs de configuration documentés                 | Cela devrait garder les noms et les formes de valeur stables, sauf si les notes de publication annoncent un changement |
| Codes de diagnostic indiqués dans la documentation | Elles devraient rester reconnaissables pour que les suppressions et les rapports de correction restent utiles          |
| API Rust crate publiées                            | Suivez le contrat de dépréciation par caisse ci-dessous                                                                |
| Composants internes de la caisse Rust non exportés | Peut changer sans prise en charge de la migration avant que la version 1 ne soit stable                                |
| Code généré et sortie TS virtuelle                 | Peut changer selon les besoins de la correction, de la compatibilité, des performances ou des diagnostics              |

## Runtime Support

Le plancher par défaut Node.js pour les paquets d’exécution npm publics est Node 22, incluant
`oxlint-plugin-vize`. Le plugin Oxlint déclare `^22 || >= 24` donc le nœud 22 et le nœud 24 ou plus récent sont
autorisés tandis que le nœud 23 reste en dehors de la matrice de compatibilité testée.

Le flux de travail de release développe des paquets natifs pour macOS, Linux et Windows sur x64 et arm64
où le paquet déclare le support. Les tâches de compatibilité CI couvrent le plancher Node déclaré ainsi que la version
actuelle du projet Node.

La matrice de fumée entièrement fraîche (`.github/workflows/native-smoke.yml`) fonctionne sur une cadence hebdomadaire
et à la demande, pas à chaque poussée de relations publiques. Il exerce le chemin d’installation de paquets publié sur
runners hébergés sur GitHub pour linux-x64-gnu, linux-arm64-gnu, darwin-arm64 et win32-x64-msvc ; Les
cibles restantes Darwin-x64 et Win32-ARM64-MSVC restent sur des runners hébergés spécifiques à l’architecture.
La matrice s’exécute contre les Nœuds 22 et 24. Les balises de libération restent bloquées par le flux de travail de libération
la fumée d’installation de tarball avant la publication des packages npm. Les vérifications de fumée en temps de fonctionnement `vize --version`,
`vize check`, `@vizejs/native` à travers `require` et `import`, et une
`@vizejs/vite-plugin` `vite build` à partir des billes de bitume installées.

Deux cibles musl Linux déclarées ne sont actuellement pas exercées par un runner hébergé à installer à nouveau.
Ils sont couverts par des artefacts de construction par plateforme ainsi que par le résolveur de dépendance optionnel `@vizejs/native-*`
jusqu’à ce qu’une fumée Alpine conteneurisée puisse localiser le
bitball natif correspondant :

| Cible            | Écart entre coureurs hôtes                                                          | Couverture rémunératrice                                                                         |
| ---------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Linux-x64-musl   | Aucune VM Alpine/musl hébergée sur GitHub n’est disponible en tant que runner natif | Le chantier de construction émet la boule de goudron en mousse ; Manuel, `node:alpine` la fumée. |
| linux-arm64-musl | Les runners hébergés sur Arm64 sont Ubuntu GNU, pas des hôtes natifs Alpine/musl    | Le projet de construction émet la boule de bitume arm64 en mousse ; manuel Alpine arm64 fumée.   |

La fermeture de ces écarts est suivie parallèlement à [#493](https://github.com/ubugeeei-prod/vize/issues/493).

La version minimale prise en charge de Rust (MSRV) pour l’espace de travail est déclarée en `Cargo.toml` sous
`[workspace.package].rust-version`. La chaîne d’outils de développement épinglée par `rust-toolchain.toml`
peut être la même version ou plus récente. Avant que la version 1 ne soit stable, le MSRV peut avancer dans n’importe quelle pré-sortie ;
le changement est mentionné dans les notes de sortie lorsqu’il change. Les packagers en aval devraient lire
`rust-version` à partir de la `Cargo.toml` d’une caisse plutôt que de les déduire à partir du fichier de la chaîne d’outils.

## Paliers de support des paquets

| Paliers                    | Paquets                                                                                       | Contrat                                                                                                                                  |
| -------------------------- | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Alpha-supporté             | `vize`, `@vizejs/native`, `@vizejs/vite-plugin`                                               | Destiné aux premiers essais de production avec des changements de casse soutenus par la note de sortie.                                  |
| Aperçu de la compatibilité | `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, `@vizejs/musea-nuxt`             | On s’attend à ce qu’il fonctionne pour des configurations hôtes courantes, mais la compatibilité hôte-framework peut évoluer rapidement. |
| Expérimental               | `oxlint-plugin-vize`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, `@vizejs/wasm` | Les paquets publics, mais les API, commandes, sortie et forme du workflow peuvent changer pendant l’alpha.                               |
| Incubation                 | `@vizejs/fresco`, `@vizejs/fresco-native`, extensions d’éditeur                               | Utile pour le développement et les retours, mais pas encore dans la cible de production alpha v1.                                        |

## Niveaux de support de caisse de rouille

Ce tableau est le contrat de compatibilité canonique pour les consommateurs crates.io. Il inclut toutes les caisses
dont les métadonnées Cargo permettent la publication, y compris les caisses temporairement différées par la publication
éditeur pendant que leur première crates.io est préparée. Les modules privés et les détails
implémentation ne sont pas des surfaces de compatibilité.

<!-- rust-crate-support:start -->

| Caisse               | Paliers                    | Public visé                                            | Point d’entrée publique                         | Suppression / dépréciation                             |
| -------------------- | -------------------------- | ------------------------------------------------------ | ----------------------------------------------- | ------------------------------------------------------ |
| `vize_carton`        | Alpha-supporté             | Auteurs du compilateur et de la bibliothèque Vize      | `vize_carton::{Allocator, Bump, FxHashMap}`     | Un mineur avec `#[deprecated]`                         |
| `vize_relief`        | Alpha-supporté             | Auteurs de l’AST et de l’intégration des compilateurs  | `vize_relief::{RootNode, CompilerOptions}`      | Une mineure avec `#[deprecated]`                       |
| `vize_armature`      | Alpha-supporté             | Outils qui analysent les modèles Vue                   | `vize_armature::{parse, Parser, Tokenizer}`     | Un mineur avec `#[deprecated]`                         |
| `vize_croquis`       | Aperçu de la compatibilité | Auteurs d’outils sémantiques et sensibles aux types    | `vize_croquis::{Croquis, Drawer}`               | Une mineure avec `#[deprecated]`                       |
| `vize_croquis_cf`    | Expérimental               | Expériences d’analyse globale du projet à adhésion     | `vize_croquis_cf::CrossFileAnalyzer`            | Pas de minimum ; Coupures de note quand c’est pratique |
| `vize_atelier_core`  | Alpha-supporté             | Auteurs backend de compilateurs personnalisés Vue      | `vize_atelier_core::{transform, generate}`      | Une mineure avec `#[deprecated]`                       |
| `vize_atelier_dom`   | Alpha-supporté             | Intégrations de compilateur et de bundler VDOM         | `vize_atelier_dom::compile_template`            | Un mineur avec `#[deprecated]`                         |
| `vize_atelier_vapor` | Expérimental               | Intégrations Opt-in du compilateur Vapor               | `vize_atelier_vapor::compile_vapor`             | Pas de minimum ; Coupures de note quand c’est pratique |
| `vize_atelier_ssr`   | Aperçu de la compatibilité | Auteurs de l’intégration SSR et framework              | `vize_atelier_ssr::compile_ssr`                 | Un mineur avec `#[deprecated]`                         |
| `vize_atelier_sfc`   | Alpha-supporté             | Auteurs d’outils SFC et de bundlers                    | `vize_atelier_sfc::{parse_sfc, compile_sfc}`    | Une mineure avec `#[deprecated]`                       |
| `vize_atelier_jsx`   | Aperçu de la compatibilité | Auteurs du compilateur et des outils JSX/TSX           | `vize_atelier_jsx::{compile_jsx, lower_source}` | Une mineure avec `#[deprecated]`                       |
| `vize_musea`         | Expérimental               | Galeries et outils de documentation de Musea           | `vize_musea::{parse_art, transform_to_csf}`     | Pas de minimum ; Coupures de note quand c’est pratique |
| `vize_fresco`        | Incubation                 | Expériences TUI                                        | `vize_fresco::{RenderTree, LayoutEngine}`       | Pas de minimum                                         |
| `vize_canon`         | Aperçu de la compatibilité | Intégrations de vérificateur de caractères et éditeurs | `vize_canon::{type_check_sfc, TypeChecker}`     | Un mineur avec `#[deprecated]`                         |
| `vize_patina`        | Aperçu de la compatibilité | Intégrations de Linter et Oxlint                       | `vize_patina::{lint, Linter}`                   | Un mineur avec `#[deprecated]`                         |

<!-- rust-crate-support:end -->

Chaque caisse enregistre également son niveau dans `package.metadata.vize.stability`. CI compare ces valeurs de métadonnées Cargo
, ce tableau, ainsi que l’ensemble complet de caisses de publication de publication, de sorte que l’addition, la suppression ou la reclassification
une caisse publiable ne peut pas changer silencieusement le contrat.

### Interprétation de la porte SemVer

`cargo-semver-checks` s’exécute pour les caisses de l’éditeur de la version qui ont un registre résoluble
des bases de base. Une caisse en attente de sa première publication, ou bloquée sur une, rejoint cette matrice dès que sa
base est disponible. Jusqu’à ce moment-là, la vérification des métadonnées/tableau/liste de sortie s’applique toujours.

| Paliers                                  | Interprétation de l’IC                                                                                                                             |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Alpha supporté / Aperçu de compatibilité | Une rupture API doit être corrigée ou suivre la fenêtre de dépréciation de la politique de support et porter un marqueur de rupture conventionnel. |
| Expérimental                             | La porte capte un déplacement accidentel ; Une interruption intentionnelle peut utiliser un marqueur de rupture sans fenêtre de dépréciation.      |
| Incubation                               | La même détection s’applique, mais l’API entière ou la caisse peut être remplacée ou supprimée dans n’importe quelle version.                      |

Les marqueurs de rupture reconnus par CI sont un `!` dans le titre de changement classique ou un pied de pied de
`BREAKING CHANGE:`. Passer la porte avec l’un ou l’autre marqueur ne fait pas annuler la fenêtre de dépréciation
pour les caisses supportées par alpha ou pour la prévisualisation de compatibilité.

## Qu’est-ce qui compte comme suffisamment stable pour Alpha

Un package ou une commande peut passer dans la couche prise en charge alpha lorsqu’elle possède :

- Itinéraires d’installation et d’utilisation documentés
- Couverture CI pour la compilation, l’installation et le support du Node en temps d’exécution
- publiez une couverture fumée pour les entrées publiées
- un propriétaire clair pour les régressions et les rapports de compatibilité
- Comportement non supporté connu documenté dans le guide concerné

## Ce qui n’est pas encore promis

L’alpha ne garantit pas une compatibilité totale avec chaque cas de contour des compilateurs Vue, chaque disposition de package
manager, chaque capacité d’édition, ni toute intégration de framework. Lorsque Vize n’est pas d’accord avec
'outil officiel de Vue, considérez la sortie officielle comme la base de compatibilité, sauf si un guide Vize
documente explicitement un comportement différent. Le compilateur bloqueant la libération, la vérification de type, l’exécution, les surfaces de compilation
et Vite sont nommés dans les
[Vue parity matrix](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/vue-parity-matrix.md).

Pour la gestion de la sécurité, voir le dépôt `SECURITY.md`. Pour les flux de travail de contribution et de correction, voir
`CONTRIBUTING.md`.
