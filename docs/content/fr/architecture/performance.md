---
title: Performances
---

<!-- Generated translation; source: architecture/performance.md -->

# Performances

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les chiffres de référence proviennent des constructions de développement et peuvent évoluer.

Vize réalise des améliorations de performance significatives par rapport au compilateur standard Vue basé sur JavaScript en tirant parti des abstractions sans coût de Rust et du multithreading natif. La vitesse n’est pas un atout agréable — c’est un prérequis pour une expérience développeur.

## Environnement de référence

Les numéros historiques ci-dessous ont été capturés sur un poste de travail local. Pour des numéros de
hébergés par CI reproductibles, adaptés aux notes de version et aux mises à jour de la documentation, utilisez les
[Blacksmith benchmark snapshot](./performance-blacksmith) générés par le workflow Tool Benchmark.

|             |                                              |
| ----------- | -------------------------------------------- |
| **Machine** | MacBook Pro (M2 Max, 12 cœurs, 96 Go de RAM) |
| **OS**      | macOS 15.3.2 (Darwin 24.3.0)                 |
| **Node.js** | v24.14.0                                     |
| **Vite**    | v8.0.0 (Descente)                            |
| **Vue**     | v3.6.0-beta.10                               |

## Benchmark : 15 000 fichiers SFC

Compilation **de 15 000 fichiers SFC Vue** (36,9 Mo au total) :

|                                | @vue/compilateur-sfc | Vize   | Accélération |
| ------------------------------ | -------------------- | ------ | ------------ |
| **Fil unique**                 | 9,35s                | 3,47s  | **2,7x**     |
| **Multi Threads**              | 4,08s                | 353 ms | **11,6x**    |
| **compiler-sfc ST vs Vize MT** | 9,35s                | 353 ms | **26,0x**    |

L’amélioration monothread vient des abstractions à coût zéro de Rust (pas de GC, pas de réchauffement JIT, disposition mémoire compatible avec le cache). L’amélioration multithread provient du pool de threads de Rayon, qui vole du travail, et qui évolue presque linéairement avec le nombre de cœurs du CPU.

### Comportement natif de mise à l’échelle par lots

| Dossiers | Batch Vize (1 fil de discussion) | Batch Vize (12 fils) | Accélération parallèle |
| -------- | -------------------------------- | -------------------- | ---------------------- |
| 100      | 25 ms                            | 3ms                  | 8,5x                   |
| 1,000    | 243 ms                           | 26 ms                | 9,4x                   |
| 5,000    | 1,25 s                           | 128 ms               | 9,7x                   |
| 15,000   | 3,75s                            | 373 ms               | 10,1x                  |

Ces numéros de lot natifs incluent les lectures de fichiers. Les petits lots, dominés par des frais fixes ; Les lots plus importants se fixent autour de 10 fois la vitesse parallèle sur cette machine à 12 cœurs.

## Pourquoi Rust ?

### Abstractions à coût zéro

Le modèle de propriété de Rust élimine les pauses de collecte des ordures. Le compilateur traite les nœuds AST via l’allocation d’arène (`vize_carton`), évitant les allocations de tas par nœud. Cela signifie :

- **Pas de pauses GC** — Dans les compilateurs basés sur V8, la collecte des déchets peut provoquer des pics de latence imprévisibles. Vize n’a aucun overhead de GC.
- **Pas de réchauffement JIT** — le compilateur JIT de V8 a besoin de temps pour optimiser les chemins chauds. Vize fonctionne à pleine vitesse dès la première instruction.
- **Performance prévisible** — La compilation anticipée de Rust signifie que la performance est cohérente entre les exécutions, sans dépendre des heuristiques d’optimisation de la V8.

### Multi-threading natif

Vize utilise [Rayon](https://docs.rs/rayon) pour la compilation parallèle de données. Chaque fichier SFC est compilé indépendamment, ce qui rend la charge de travail embarrassante et parallèle. Le planificateur de vol de travail de Rayon garantit une utilisation optimale des cœurs :

```rust
// Simplified: parallel compilation of all .vue files
files.par_iter().map(|file| {
    let arena = Bump::new();
    let ast = parse(file, &arena);
    let analyzed = analyze(ast, &arena);
    compile(analyzed, &arena)
}).collect()
```

L’approche de vol de travail signifie que si un fichier est significativement plus grand que les autres, les threads inactifs voleront du travail dans la file d’attente du thread occupé, maintenant un équilibrage de charge quasi parfait.

### Disposition efficace de la mémoire

La disposition des structures de Rust et les discriminants d’enum sont compacts. La représentation AST dans `vize_relief` est compatible avec le cache, réduisant les goulots d’étranglement en bande passante mémoire :

- **Discriminants d’énume** — Les énums rouillés sont dimensionnés selon le plus petit type qui correspond au discriminant. Un `NodeKind` avec 20 variantes utilise un seul octet, et non une chaîne allouée au tas.
- **Empaquetage de struct** — Rust réorganise automatiquement les champs de struct pour un alignement optimal, minimisant ainsi les octets de bourrage.
- **Pas d’en-têtes d’objet** — Contrairement aux objets JavaScript (qui transportent des chaînes prototypes, des cartes de propriétés et des pointeurs de classe cachés), les structs Rust sont des données pures sans aucune surcharge.

### Pas de surcharge en durée de fonctionnement

Contrairement aux compilateurs basés sur JavaScript qui s’exécutent dans la version 8, Vize compile directement en code natif. Il n’y a pas d’échauffement JIT, pas de ramassage d’ordures, et pas de contention pour les boucles d’événement. Le binaire du compilateur est un exécutable unique, lié statiquement, qui démarre et s’exécute à pleine vitesse.

## Choix architecturaux pour la performance

### Allocation des arènes

`vize_carton` fournit un allocateur de bump pour les nœuds AST utilisant [bumpalo](https://docs.rs/bumpalo). Cela signifie :

- **L’allocation est O(1)** — Il suffit de pousser un pointeur vers l’avant. Pas de parcours libre de listes, pas de gestion de fragmentation.
- **La délocation est O(1)** — Abandonner toute l’arène d’un coup une fois la compilation terminée. Pas de surcharge de déallocation par nœud.
- **La localité mémoire est excellente** — les nœuds sont compactés de façon contiguë en mémoire, maximisant les impacts du cache L1/L2 lors de la traversée de l’arbre.

C’est un avantage fondamental par rapport au collecteur d’ordures générationnel de V8, qui doit tracer périodiquement des objets accessibles et compacter la mémoire.

### Streaming Tokenizer

Le tokenizer de `vize_armature`traite les entrées sous forme d’un flux d’octets, évitant ainsi la nécessité de construire des tableaux intermédiaires de jetons. L’analyseur consomme les jetons paresseusement — chaque jeton est produit à la demande et immédiatement consommé. Cela réduit l’utilisation maximale de la mémoire et améliore le comportement du cache.

### Stage en cordes

Les chaînes courantes (noms de directives, noms d’attributs, noms de balises HTML) sont intégrées via `compact_str` et tables de hachage parfaites (`phf`). Cela signifie :

- La comparaison de chaînes est une comparaison de pointeurs (O(1)) au lieu d’une comparaison caractère par caractère (O(n))
- Les chaînes dupliquées partagent une seule allocation
- Les recherches de hachage pour les chaînes connues sont calculées au moment de la compilation

### Compilation incrémentale

Le plugin Vite (`@vizejs/vite-plugin`) utilise la mise en cache au niveau des fichiers. Seuls les fichiers modifiés sont recompilés pendant le développement, minimisant ainsi la latence du HMR. La clé de cache est le hachage du contenu du fichier, garantissant que les fichiers non modifiés ne soient jamais recompilés.

## Benchmark : Linter — patina vs eslint-plugin-vue

Linting **15 000 fichiers Vue SFC** :

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

Formatage **de 15 000 fichiers Vue SFC** :

|           | Plus jolie (CLI) | Glyphe de Vize (ST) | Accélération | Glyphe Vize (MT) | **Cli plus joli vs Vize MT** |
| --------- | ---------------- | ------------------- | ------------ | ---------------- | ---------------------------- |
| **Temps** | 101,20s          | 2,97 s              | **34,1x**    | 835 ms           | **121,2x**                   |

Courez `vp run --workspace-root bench:fmt` pour vous reproduire.

## Benchmark : Type Checker — canon vs vue-tsc

Vérification de type **500 fichiers SFC Vue générés** avec le chemin de diagnostic actuel soutenu par Corsa :

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
