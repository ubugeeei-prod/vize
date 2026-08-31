---
title: Performances
---

<!-- Generated translation; source: architecture/performance.md -->

# Performances

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les chiffres de référence proviennent des constructions de développement et peuvent évoluer.

Vize réalise des améliorations de performance significatives par rapport au compilateur standard Vue basé sur JavaScript en tirant parti des abstractions sans coût de Rust et du multithreading natif. La vitesse n’est pas un atout agréable — c’est un prérequis pour une expérience développeur.

## Environnement de référence

Deux environnements de mesure apparaissent sur cette page, et chaque chiffre ci-dessous précise
duquel il provient.

**Runner de référence.** Les comparaisons entre outils sont mesurées par le workflow Tool Benchmark
et versionnées dans `bench/results/tool-benchmark-latest.json`. Cet artefact est la source citable,
et l'[instantané de benchmark Blacksmith](./performance-blacksmith) le publie intégralement.

|                |                                                       |
| -------------- | ----------------------------------------------------- |
| **Machine**    | `blacksmith-32vcpu-ubuntu-2404` (32 vCPU, AMD EPYC)   |
| **Instantané** | commit `1511788d96ea`, 2026-07-30                     |
| **Méthode**    | médiane de 5 exécutions mesurées après 1 préchauffage |
| **Versions**   | vize 0.303.0 · vue 3.6.0-beta.10 · Node v24.14.0      |

**Poste de travail local.** Les tableaux du linter, du formateur et du vérificateur de types plus
bas sont encore maintenus à la main à partir de benchmarks locaux (`bench/lint.ts`, `bench/fmt.ts`,
`bench/check.ts`) et ont été mesurés ici. Ils ne sont pas encore reproductibles sur le runner de
référence : lisez-les comme des indications.

|             |                                           |
| ----------- | ----------------------------------------- |
| **Machine** | MacBook Pro (M2 Max, 12 cœurs, 96 Go RAM) |
| **OS**      | macOS 15.3.2 (Darwin 24.3.0)              |
| **Node.js** | v24.14.0                                  |
| **Vite**    | v8.0.0 (Rolldown)                         |
| **Vue**     | v3.6.0-beta.10                            |

## Benchmark : 15 000 fichiers SFC

Compilation de **15 000 fichiers Vue SFC générés** (58,7 Mo au total) sur le runner de référence :

|                              | @vue/compiler-sfc | Vize    | Accélération |
| ---------------------------- | ----------------- | ------- | ------------ |
| **Mono-thread**              | 17,15s            | 3,95s   | **4,3x**     |
| **Tous les cœurs (32 vCPU)** | 6,08s             | 329,2ms | **18,5x**    |
| **compiler-sfc 1T vs max**   | 17,15s            | 329,2ms | **52,1x**    |

Source : la surface `compile` de l'instantané versionné `bench/results/tool-benchmark-latest.json`
([run 30557718030](https://github.com/ubugeeei-prod/vize/actions/runs/30557718030)) — le même
artefact que publient `README.md` et l'[instantané de benchmark
Blacksmith](./performance-blacksmith).

L'amélioration mono-thread provient des abstractions à coût zéro de Rust (pas de GC, pas de préchauffage JIT, disposition mémoire favorable au cache). L'amélioration multi-thread provient du pool de threads à vol de travail de Rayon, qui passe à l'échelle avec le nombre de cœurs.

> **Note :** cet instantané a été pris à la version vize 0.303.0, avant le travail sur l'arène et les expressions décrit dans « Choix architecturaux pour la performance ». Il est daté et reproductible, mais il ne mesure pas l'arbre actuel. Le ré-enregistrement des surfaces inter-outils sur le runner de référence reste à faire.

## Pourquoi Rust ?

### Abstractions à coût zéro

Le modèle de propriété de Rust élimine les pauses de ramasse-miettes. Les nœuds de l'AST du template vivent dans une arène propre à chaque compilation (`vize_carton`) et empruntent leur texte au code source du template : un nœud est donc de la donnée pure, sans allocation de tas qui lui appartienne (`crates/vize_relief/src/relief/elements.rs`). Cela signifie :

- **Pas de pauses GC** — Dans les compilateurs basés sur V8, le ramasse-miettes peut provoquer des pics de latence imprévisibles. Vize n'a aucun surcoût de GC.
- **Pas de préchauffage JIT** — Le compilateur JIT de V8 a besoin de temps pour optimiser les chemins chauds. Vize fonctionne à pleine vitesse dès la première instruction.
- **Performance prévisible** — La compilation anticipée de Rust signifie que la performance est cohérente d'une exécution à l'autre, sans dépendre des heuristiques d'optimisation de V8.

### Multi-threading natif

Vize utilise [Rayon](https://docs.rs/rayon) pour la compilation parallèle de données. Chaque fichier SFC est compilé indépendamment, ce qui rend la charge de travail trivialement parallèle, et le pilote de lot dans `crates/vize/src/commands/build/runner.rs` répartit les entrées planifiées sur le pool :

```rust
// crates/vize/src/commands/build/runner.rs — le pilote de lot
planned_inputs
    .par_iter()
    .map(|input| compile_file_with_profile(&input.source, compile_settings, &stats))
    .collect()
```

L'arène n'est pas créée ici. Elle est acquise là où elle naît — aux points d'entrée template, script et style à l'intérieur de `vize_atelier_sfc` — depuis un pool propre à chaque worker :

```rust
// par exemple crates/vize_atelier_sfc/src/compile.rs
let allocator = vize_carton::pool::acquire();
```

L'approche par vol de travail signifie que si un fichier est nettement plus gros que les autres, les threads inactifs voleront du travail dans la file du thread occupé, maintenant un équilibrage de charge quasi parfait.

### Disposition mémoire efficace

La disposition des structures et les discriminants d'énumération de Rust sont compacts. La représentation de l'AST dans `vize_relief` est favorable au cache, ce qui réduit les goulets d'étranglement de bande passante mémoire :

- **Discriminants d'un octet** — `NodeType` est un `#[repr(u8)]` à 27 variantes (`crates/vize_relief/src/relief/core.rs`) : le type d'un nœud coûte un octet, et non une chaîne allouée sur le tas.
- **Tailles de nœuds figées** — chaque nœud de template porte une assertion `const` sur sa taille, si bien qu'un champ qui fait grossir un nœud casse la compilation plutôt que le budget. `ElementNode` fait 104 octets, `SimpleExpressionNode` 88, `AttributeNode` 56, `TextNode` 24 et `SourceLocation` 8 (`crates/vize_relief/src/relief/{elements,expressions,control_flow,nodes}.rs`).
- **Pas d'en-têtes d'objet** — Contrairement aux objets JavaScript (qui transportent des chaînes de prototypes, des tables de propriétés et des pointeurs de classe cachée), les structures Rust sont de la donnée pure, sans surcoût.

### Pas de surcoût d'exécution

Contrairement aux compilateurs basés sur JavaScript qui s'exécutent dans V8, Vize compile directement en code natif. Il n'y a pas de préchauffage JIT, pas de ramasse-miettes et pas de contention de boucle d'événements. La CLI est livrée sous forme d'exécutable natif autonome par plateforme — entièrement statique sur les cibles Linux musl, ce que la CI vérifie (`tools/commands/ci/github/verify-musl-cli-binary.rs`), et liée dynamiquement à la bibliothèque C du système sur les cibles glibc, macOS et Windows. Le plugin Vite charge le même compilateur comme module natif Node (`@vizejs/native`) plutôt que comme processus séparé.

## Choix architecturaux pour la performance

### Allocation par arène

`vize_carton::Allocator` est un allocateur à pointeur glissant pour les nœuds de l'AST : il enveloppe [`oxc_allocator`](https://docs.rs/oxc_allocator) afin que les nœuds de template et les expressions JavaScript retenues partagent une seule arène et une seule durée de vie (`crates/vize_carton/src/allocator.rs`). Cela signifie :

- **L'allocation est en O(1)** — Il suffit d'avancer un pointeur. Pas de parcours de liste libre, pas de gestion de fragmentation.
- **La récupération est en O(1) et réutilisée** — À la fin d'une compilation, l'arène est remise à zéro par `reset()` et non détruite : le pointeur revient au début du bloc et l'arène retourne dans une liste libre propre au worker (`crates/vize_carton/src/pool.rs`, plafonnée à 4 arènes inactives par worker). Le fichier suivant réutilise la même mémoire au lieu d'en redemander au système.
- **La localité mémoire est excellente** — Les nœuds sont tassés de façon contiguë en mémoire, ce qui maximise les succès de cache L1/L2 lors du parcours de l'arbre.

Les valeurs adossées à l'arène ne peuvent pas survivre à leur compilation. Ce contrat est imposé par le compilateur (`reset` prend `&mut self`, et le garde du pool possède son arène) et, dans les builds de débogage, par un marqueur de génération qui panique si une valeur est lue après le recyclage de son arène (`crates/vize_carton/src/allocator/generation.rs`).

Rien dans l'AST n'implémente `Drop` — les types conteneurs de l'arène refusent les charges utiles nécessitant une destruction, ce qui en fait une erreur de compilation et non une convention.

### Tokeniseur en une passe

Le tokeniseur de `vize_armature` est une machine à états orientée octets sur `&[u8]` (`crates/vize_armature/src/tokenizer.rs`). Il ne matérialise jamais de jeton : il n'existe ni type `Token` ni vecteur de jetons nulle part dans le compilateur. À la place, `tokenize()` effectue une passe jusqu'à la fin de l'entrée et pousse des événements vers un collecteur `Callbacks` que le parseur implémente — chaque événement est donc traité de façon synchrone au moment où il est produit, et le tableau intermédiaire qu'exigerait une conception en deux phases n'existe tout simplement pas.

Notez qu'il s'agit d'un fonctionnement par poussée, et non d'une lecture paresseuse : le parseur ne demande pas de jetons et ne peut pas interrompre la boucle en cours de route.

### Internement de chaînes

Les noms qui reviennent au sein d'une compilation — noms de directives normalisés, noms d'assets, noms d'arguments en camelCase — sont internés en atomes adossés à l'arène par `vize_carton::interner`, avec un ensemble [`phf`](https://docs.rs/phf) calculé à la compilation de 181 noms bien connus (balises HTML/SVG/MathML, composants intégrés de Vue, noms de directives et attributs traités spécialement par les transformations) qui se résolvent en littéraux `'static` sans toucher l'arène. Cela signifie :

- Les noms calculés répétés partagent une seule allocation dans l'arène
- Les recherches de noms bien connus utilisent un hachage parfait calculé à la compilation, sans allocation

L'internement est la solution de repli, pas le cas courant. La plupart des noms ne sont jamais copiés : un nom de balise, un nom d'attribut et l'essentiel du contenu des expressions sont des tranches `&'a str` empruntées directement au code source du template, si bien que le chemin courant n'alloue rien (`crates/vize_carton/src/interner.rs` documente la politique champ par champ).

Les atomes sont de simples `&'a str` : les comparaisons de noms sont donc des comparaisons de contenu, et non d'identité de pointeur. L'internement apporte des économies d'allocation et de la localité de cache — ce n'est pas un raccourci pour `==`.

### Compilation incrémentale

Le plugin Vite (`@vizejs/vite-plugin`) met en cache au niveau du fichier, en deux couches aux clés différentes :

- **En mémoire, pour le dev et le HMR** — indexé par le chemin de fichier résolu (`npm/builder/vite/src/plugin/compiled-module-cache.ts`). Les entrées sont explicitement évincées lors d'une mise à jour à chaud plutôt que ré-indexées : un fichier modifié est recompilé, ses voisins non.
- **Détection de changement à la pré-compilation** — indexée par `mtime` + taille, comparés côté Rust (`crates/vize_atelier_sfc/src/vite_plugin/precompile.rs`). C'est ce filtre qui décide quels fichiers un lot recompile.
- **Sur disque, entre processus** — `node_modules/.vize/vite-precompile`, indexé par un hachage SHA-256 de la source plus une clé de manifeste couvrant l'identité du binaire du compilateur et les options résolues (`npm/builder/vite/src/plugin/precompile-cache-key.ts`). Le hachage de contenu est utilisé ici précisément parce que `mtime` n'est pas fiable d'une machine ou d'un checkout à l'autre.

## Mesuré : travail sur l'arène et les expressions

Le travail sur les entrailles du compilateur décrit ci-dessus est mesuré par un harnais de micro-benchmarks par crate (`cargo bench --bench davinci`) sur une échelle fixe de six fixtures, `benchmarks/davinci_harness/fixtures/{small,medium,large,stress-deep,stress-wide,stress-interp}.vue`.

**Comment lire ces chiffres.** Les comptes d'allocations sont déterministes et indépendants de la machine : ce sont donc des faits exacts, et ils servent de cliquet anti-régression. Les temps d'exécution ont été relevés sur une machine de développement partagée avec l'échantillonnage `--quick` et sont **seulement indicatifs** — les enregistrements sur le runner de référence (Blacksmith) restent à faire, ce qui explique que chaque entrée `wall_p50_ns` et `allocs` de `davinci-road/plan/budgets.toml` vaille encore `0`, c'est-à-dire « pas encore enregistré, informatif seulement ». Les fichiers de résultats de chaque exécution atterrissent dans `bench/results/davinci/` : ce sont des artefacts locaux, pas des références versionnées.

Appels d'allocation par compilation, avant et après le travail sur les chaînes et l'arène (exacts, mêmes fixtures) :

| Fixture         | Analyse   | Compilation DOM | Compilation SSR | Compilation Vapor |
| --------------- | --------- | --------------- | --------------- | ----------------- |
| `small`         | 21 → 9    | 52 → 39         | 73 → 60         | 90 → 73           |
| `medium`        | 171 → 107 | 329 → 264       | 1 099 → 1 030   | 588 → 515         |
| `large`         | 350 → 272 | 656 → 573       | 1 106 → 983     | 1 136 → 1 003     |
| `stress-deep`   | 397 → 155 | 669 → 426       | 612 → 369       | 764 → 514         |
| `stress-wide`   | 213 → 204 | 255 → 245       | 416 → 405       | 280 → 261         |
| `stress-interp` | 616 → 105 | 1 048 → 536     | 3 149 → 2 637   | 1 495 → 974       |

Les tailles de nœuds ont diminué en conséquence, et les nouvelles tailles sont figées par des assertions `const` : `RootNode` 296 → 224 octets, `DirectiveNode` 208 → 176, `ElementNode` 128 → 104, `SimpleExpressionNode` 120 → 88, `AttributeNode` 80 → 56, `TextNode` 32 → 24.

**Pic de mémoire résidente.** La réutilisation de l'arène d'un fichier à l'autre est le gain isolé le plus important, et c'est un résultat de mémoire, pas de vitesse. Compilation des 36 541 SFC du corpus versionné (`vize build "tests/_fixtures/_git/**/*.vue" --format stats`, binaires `ci-opt`, taille maximale de l'ensemble résident via `/usr/bin/time -l`, même machine avant et après) :

| Workers | Avant    | Après    | Écart       | Exécutions |
| ------- | -------- | -------- | ----------- | ---------- |
| 12      | 766,5 Mo | 171,1 Mo | **−77,7 %** | 5          |
| 1       | 717,0 Mo | 88,2 Mo  | **−87,7 %** | 3          |

Le chiffre à un seul worker est le signal d'accumulation : il ne dépend pas de l'ordonnancement et montre donc que l'ancien pic venait d'une fuite par fichier, et non des arènes par worker. Le temps d'exécution est resté inchangé dans le bruit, et les 36 541 fichiers émis étaient identiques octet pour octet (manifestes SHA-256 comparés).

**Ré-analyse des expressions.** Les expressions de template ne sont désormais analysées qu'une seule fois, pendant l'analyse du template, et sont retenues sur le nœud. Les consommateurs lisent l'AST retenu au lieu de ré-analyser le texte. Sur la voie SSR, la fixture `stress-interp` est passée de 500 ré-analyses d'expressions redondantes par compilation à zéro, et cette voie fusionnée affiche un gain net de **−13,6 %** en temps d'exécution par rapport à l'arbre antérieur à la rétention (346,8µs → 299,8µs) — l'analyse coûte désormais plus cher et les consommateurs beaucoup moins. Les voies DOM et Vapor n'avaient aucune ré-analyse à supprimer sur cette fixture : elles supportent donc encore le coût d'analyse ajouté. Combler cet écart est suivi comme un reste de travail de phase, et non comme un gain déjà livré.

## Benchmark : Linter — patina vs eslint-plugin-vue

Analyse (lint) de **15 000 fichiers Vue SFC**, poste de travail local :

|           | eslint-plugin-vue (ST) | Patine Vize (ST) | Accélération | eslint-plugin-vue (MT) | Patine Vize (MT) | Accélération | **eslint ST vs Vize MT** |
| --------- | ---------------------- | ---------------- | ------------ | ---------------------- | ---------------- | ------------ | ------------------------ |
| **Temps** | 45,08s                 | 4,02             | **11,2x**    | 16,38s                 | 784 ms           | **20,9x**    | **57,5x**                |

Courez `vp run --workspace-root bench:lint` pour vous reproduire.

### Profil de peluches sensible au type

Le linting sensible au type est intentionnellement profilé aux phases où le coût tend à se regrouper : analyse SFC, analyse
Croquis, génération virtuelle de TypeScript, collecte de requêtes de modèles et sondes Corsa. Lorsque
plusieurs règles sensibles au type appuyées sur des modèles sont activées, Patina collecte les requêtes d’expression et de promesses de
modèles lors d’une seule marche AST avant la phase de sonde Corsa. La collection de requêtes partage également
l’analyse d’expressions OXC pour les vérifications de template non sécurisé et de promesse flottante, de sorte qu’une expression de modèle
ne paie pas le coût d’analyse dupliqué lorsque les deux règles sont activées.

Faites `vize lint --profile --preset opinionated src` pour voir ces rangées dans un projet local. Le rapport de profil
comprend également une section d’audit stricte qui vérifie la couverture du temps de clôture, le temps cumulatif
travailleur, les impacts à seuils lents et les périodes internes capturées avant de lister les fichiers chauds et les opérations de
internes. Les lignes à chaud affichent la part et le débit par stade, et les lignes d’opération signalent les
dominantes ou les pics max/moyens.

## Benchmark : Formateur — glyphe vs Pretty

Formatage de **15 000 fichiers Vue SFC**, poste de travail local :

|           | Plus jolie (CLI) | Glyphe de Vize (ST) | Accélération | Glyphe Vize (MT) | **Cli plus joli vs Vize MT** |
| --------- | ---------------- | ------------------- | ------------ | ---------------- | ---------------------------- |
| **Temps** | 101,20s          | 2,97 s              | **34,1x**    | 835 ms           | **121,2x**                   |

Courez `vp run --workspace-root bench:fmt` pour vous reproduire.

## Benchmark : Type Checker — canon vs vue-tsc

Vérification de type de **500 fichiers Vue SFC générés** avec le chemin de diagnostic actuel soutenu par Corsa, poste de travail local :

|           | vue-tsc (ST)   | Canon Vize (ST) | Accélération       | vue-tsc (MT)   | Canon Vize (MT) | Accélération       | **vue-tsc ST vs Vize MT** |
| --------- | -------------- | --------------- | ------------------ | -------------- | --------------- | ------------------ | ------------------------- |
| **Temps** | 4,38s          | 511ms           | n/a (cross-engine) | 4,41s          | 493 ms          | n/a (cross-engine) | n/a (cross-engine)        |
| **Taux**  | 114 fichiers/s | 979 fichiers/s  |                    | 113 fichiers/s | 1.0k fichiers/s |                    |                           |

Les lignes de contrôle de type couvrent deux moteurs TypeScript : vue-tsc exécute le compilateur JavaScript tandis que Vize check exécute tsgo natif (Corsa). Aucun rapport unique n’est donc publié (`n/a (cross-engine)`) ; chaque classe de moteur est classée séparément, car un chiffre unique attribuerait la réécriture en Go de TypeScript à la couche Vue. Les deux mesures sont réelles et proviennent de la même exécution ; voir l’[instantané de benchmark Blacksmith](./performance-blacksmith) pour le classement par classe de moteur.

> **Note :** Le canon Vize est encore en phase de développement initial et la voie de diagnostic soutenue par Corsa rattrape encore la fidélité vue-tsc. Ces mesures reflètent l’implémentation native actuelle CLI-first avec un plan de secours de session de projet et évolueront à mesure que la couverture et la parité des diagnostics s’amélioreront.

Faites `node bench/check.ts 500` après `cargo build --release -p vize` pour reproduire ce benchmark rapide.

### Profil de vérification de type

Le dispositif de profil 500-SFC conserve la majeure partie du temps mural à l’intérieur de la commande CLI Corsa, tandis que le chemin rapide d’importation en réécriture supprime le coût précédent de syntase OXC pour les fichiers sans spécificateurs Vue :

| Métrique                         | Avant    | Actuel   |
| -------------------------------- | -------- | -------- |
| `canon.import.rewrite.vue`       | 26,77 ms | 2,45 ms  |
| Plus grand TS virtuel généré     | 15 401B  | 14 414B  |
| Temps total sur le mur du profil | 1,88s    | 668 ms   |
| Phase de diagnostic de Corsa     | 1,67s    | 482 ms   |
| Analyse Corsa CLI                | N/D      | 10,41 ms |

La phase de `virtual project` côté rouille — analyse SFC par fichier, analyse Croquis
génération de Virtual TS et réécriture d’importation — est déployée sur le pool
thread de rayon à l’intérieur de `VirtualProject::register_paths`. Chaque fichier `.vue` est indépendant
une fois les options de l’espace de travail résolues, donc un seul lot paralléllise
proprement. Sur un dispositif de 1 000 SFC, la phase passe de ~71 ms à ~25 ms avant même que
Corsa ne soit invoquée.

### Luminaire e2e très chargé en diagnostics

`bench/check.ts` mesure aussi l’application `tests/_fixtures/_git/npmx.dev` lorsque le luminaire est présent. Cela capture le chemin de correspondance de diagnostic sur un véritable élément d’application :

| Calendrier           | Fichiers sources SFC | Fichiers virtuels | Diagnostic | Canon Vize |
| -------------------- | -------------------- | ----------------- | ---------- | ---------- |
| npmx.dev application | 134                  | 226               | 1,053      | 1,94s      |

Le profil actuel de ce luminaire maintient l’analyse diagnostique CLI à ~7 ms. La plupart du temps est désormais dans la commande CLI de Corsa elle-même. L’auto-import des stubs du framework dans un seul fichier ambiant a également réduit le plus grand fichier Virtual TS généré d’environ 275 Ko à 144 Ko.

## Benchmark : Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue

Version Vite avec **1 000 importations SFC Vue** (toutes importées en une seule entrée) :

|                           | @vitejs/plugin-vue | @vizejs/vite-plugin | Accélération |
| ------------------------- | ------------------ | ------------------- | ------------ |
| **Temps de construction** | 1.71s              | 631.7ms             | **2.7x**     |

> Note : `@vizejs/vite-plugin` remplace uniquement l’étape de compilation Vue SFC — la différence de performance vient entièrement de cette partie. La résolution des dépendances, la construction de graphes de modules, le regroupement (Rolldown) et tous les autres internes Vite sont identiques à `@vitejs/plugin-vue`. Pour la performance purement en compilation, voir la [Compiler benchmark](#benchmark-15000-sfc-files) ci-dessus. `@vizejs/vite-plugin` pré-compile avec enthousiasme `.vue` fichiers en utilisant une compilation multithread native, ce qui permet également un HMR plus rapide.

Cette ligne est la surface `vite` de l'instantané commité `bench/results/tool-benchmark-latest.json` ([run 30557718030](https://github.com/ubugeeei-prod/vize/actions/runs/30557718030)) — le même artefact que celui publié par `README.md` et par l'[instantané de benchmark Blacksmith](/architecture/performance-blacksmith). `tests/tooling/docs-vite-benchmark-row.test.ts` la verrouille sur cet artefact, dans toutes les locales.

Le chiffre publié ici jusqu'à présent — `957ms` / `479ms` / `2.0x` — provenait de `bench/vite.ts` avant #3392, qui mesurait Vize avec un cache de pré-compilation persistant laissé chaud par son propre échauffement, tandis que `@vitejs/plugin-vue` compilait à froid. Ce harnais rapporte désormais des lignes à froid et à chaud séparées sur la machine où il s'exécute : c'est un diagnostic local, pas une accélération publiable. Utilisez `vp run --workspace-root bench:vite` pour comparer un changement à lui-même.
