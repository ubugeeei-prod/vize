---
title: Pratiques en ingénierie linguistique
---

<!-- Generated translation; source: architecture/language-engineering-practices.md -->

# Pratiques en ingénierie linguistique

Vize est une chaîne d’outils Vue, mais elle a les mêmes modes de défaillance qu’un compilateur : de minuscules modifications de syntaxe peuvent
déplacer simultanément les diagnostics, la génération de code, le comportement de l’éditeur, la sortie des paquets et
les performances. Cette page enregistre les pratiques de traitement du langage que Vize adopte à partir des dépôts matures de compilateurs et de vérificateurs de type
, puis les mappe aux fixtures, snapshots, tests de parité, benchmarks,
et portes de release de Vize.

## Signaux sources

| Source                                                                                                                                  | Pratique observée                                                                                                                                                                                                                                                                   | Traduction de Vize                                                                                                                                                                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`rust-lang/rust`](https://github.com/rust-lang/rust) et la [`rustc-dev-guide`](https://rustc-dev-guide.rust-lang.org/tests/intro.html) | `compiletest` regroupe les tests UI par suite, stocke les sorties attendues près des cas sources, utilise `tidy` pour les invariants du dépôt, et suit séparément les régressions de l’écosystème et des performances.                                                              | Considérez d’abord les changements orientés vers le compilateur comme des changements de fixation. Gardez les attentes entre parseurs/compilateurs dans `tests/fixtures` et `tests/expected`, et gardez les invariants du dépôt dans `tests/tooling/*.test.ts`.    |
| [`rustc` ecosystem and perf testing](https://rustc-dev-guide.rust-lang.org/tests/ecosystem.html)                                        | Crater, cargotest, constructeurs de grands projets et rustc-perf rendent explicites la compatibilité large et les risques de performance avant ou après la fusion des modifications du compilateur.                                                                                 | Escaladez la sémantique générale de Vue, la forme du code généré ou les changements de chemins chauds vers des équipements réels, la matrice de parité Vue et le budget de référence PR au lieu de ne vous reposer que sur les fixatures unitaires.                |
| [`rust-fuzz/cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) et libFuzzer                                                          | Les cibles fuzz guidées par la couverture exécutent des entrées d’octets arbitraires, persistent les corpus et minimisent les reproducteurs de plantage avant de les transformer en régressions déterministes.                                                                      | Les limites de parseur fuzz, lexer, CSS, expression et compilation de modèles à partir de `tests/fuzz` avec `cargo +nightly fuzz run <target>` avant de considérer les correctifs de plantage comme complets.                                                      |
| [Linux kernel testing](https://www.kernel.org/doc/html/next/dev-tools/testing-overview.html)                                            | KUnit couvre les petites unités white-box, kselftest couvre les interfaces système visibles par l’utilisateur, KCOV alimente le fuzzing guidé par couverture, et `perf stat` capture le compteur reproductible et le statut de synchronisation.                                     | Séparez les petits contrôles au niveau caisse des contrôles d’intégration CLI/espace de travail, utilisez la couverture/fuzzing pour les entrées arbitraires, et attachez le statut du profil ou du benchmark lorsque les chemins chauds bougent.                  |
| [Chromium testing and CQ](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/testing_in_chromium.md)                    | Les couches chrome : unités hermétiques, navigateur, web, télémétrie et tests de fuzzer ; Les CQ/trybots rendent explicites les voies coûteuses ou instables, et ClusterFuzz exécute des cibles fuzz découvertes à grande échelle.                                                  | Gardez les tests Vize hermétiques par défaut, escaladez le comportement du navigateur/application vers des éléments réels, utilisez le budget PR benchmark pour un statut similaire à Telemetry, et conservez les reproducteurs fuzz pour le triage.               |
| [V8 testing](https://v8.dev/docs/test) et [feature launch](https://v8.dev/docs/feature-launch-process)                                  | V8 fait tourner des suites de moteurs telles que `mjsunit` et Test262, régénère les fichiers attendus uniquement après révision, utilise les flux de comparaison de `tools/run_perf.py` et de benchmark, et nécessite un fuzzing avant la livraison des fonctionnalités du langage. | Traitez les changements de compatibilité Vue/TS comme des fonctionnalités linguistiques : citez le comportement source, ajoutez des tests de scénario, comparez les performances lorsque c’est pertinent, et effectuez ou planifiez un fuzzing avant la promotion. |
| [`microsoft/TypeScript`](https://github.com/microsoft/TypeScript)                                                                       | Le graphique de tâches de Présente-Présent sépare les tâches de construction, formatage, lint, test et baseline. La sortie du compilateur est examinée via `tests/baselines/reference` par rapport à la sortie locale générée avant `baseline-accept`.                              | Conservez des instantanés sous forme de contrats examinés. Un `tests/snapshots/*` modifié ou un instantané de `insta` Rust doit être expliqué par le PR et limité au comportement modifié.                                                                         |
| [`TypeScript tests/cases/fourslash`](https://github.com/microsoft/TypeScript/tree/main/tests/cases/fourslash)                           | Le comportement des services de langage orienté éditeur est capturé sous forme de milliers de fichiers de scénarios plutôt que déduit uniquement à partir de tests de compilation.                                                                                                  | LSP, correctif rapide, complétion, survol et modifications incrémentales de l’éditeur devraient avoir une couverture de fumée ou d’intégration au niveau du scénario, pas seulement des éléments de parser/compilateur.                                            |
| [`microsoft/typescript-go`](https://github.com/microsoft/typescript-go)                                                                 | Le port natif conserve le sous-module TypeScript comme implémentation de référence, ajoute des tests minimaux de compilateurs, écrit les sorties générées dans `testdata/baselines/local`, et considère les lignes de base réduites `.diff` comme des preuves de convergence.       | Comparez la sortie Vize avec le comportement officiel de Vue et TypeScript avant d’introduire une règle spécifique à Vize. Si Vize diverge intentionnellement, documentez la raison et le niveau de compatibilité.                                                 |
| [`facebook/flow`](https://github.com/facebook/flow)                                                                                     | Flow conserve les tests d’intégration en forme de répertoire avec `.exp` sortie attendue, supporte la réenregistrement des modifications intentionnelles de la sortie, et utilise un `newtests` de style action/assertion pour les flux d’éditeur et de serveur.                    | Je préfère les petits dispositifs de scénarios pour les diagnostics et les flux de travail de l’éditeur. Les instantanés réenregistrés ne sont acceptables qu’après avoir examiné le diff et maintenu le bruit généré hors de la ligne de base.                    |

## Classes de changement Vize

Chaque PR de traitement de langage doit nommer sa classe de changement et inclure des preuves issues de la ligne de
correspondante. Utilisez la commande la plus étroite pendant le développement, puis élargissez lorsque le changement touche le comportement
partagé.

| Changement de classe                              | Preuves requises                                                                                                                                                           | Commandes courantes                                                                                                                                        |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parseur ou AST                                    | Installation minimale du parser, AST ou sortie d’erreur attendue, et pas de rafraîchissement large d’instantané.                                                           | `cargo test -p vize_armature`, `cargo test -p vize_test_runner`, `node tests/tooling/support/generate-expected.ts <fixture>`                               |
| Compilateur et codegen                            | Fixture source minimale, sortie attendue DOM/Vapor/SSR, et parité réelle lorsque la forme du runtime émet change.                                                          | `cargo test -p vize_atelier_dom`, `cargo test -p vize_atelier_vapor`, `vp run --filter './tests' test:build`                                               |
| Analyse sémantique, lint et analyse croisée       | Règle ou dispositif d’analyseur, instantané de sortie JSON ou agent, et documentation pour les diagnostics modifiés.                                                       | `cargo test -p vize_patina`, `vp run --filter './tests' test:lint`, `node --test tests/tooling/snapshot-baselines.test.ts`                                 |
| Virtual TypeScript et vérification de type        | Installation SFC minimale, instantané de diagnostic mappé, revue virtuelle TS générée, et note officielle de parité Vue ou TypeScript.                                     | `vp run --filter './tests' test:check:fixtures`, `cargo test -p vize_canon`, `vize check --show-virtual-ts <file>`                                         |
| Formateur et LSP                                  | Sortie formatée dorée ou couverture de fumée de protocole, plus une vérification d’intégration ciblée de l’éditeur lorsque le comportement est visible pour l’utilisateur. | `cargo test -p vize_glyph`, `cargo test -p vize_maestro`, `node --test tests/tooling/lsp-smoke.test.ts`                                                    |
| Emballage à l’exécution, version ou documentation | Test de gouvernance, installation de fumée ou couverture du flux de travail, et documents de sortie/préparation lorsque la posture de production change.                   | `node --test tests/tooling/*.test.ts`, `rust-script tools/commands/release/npm/smoke-release-install.rs --prepare-manifests --runtime-checks`, `vp run --workspace-root check:ci` |

## Voies d’assurance

Certains changements nécessitent une seconde lentille en plus de la classe de changement. Ces voies rendent explicites le statut de sécurité,
le statut de performance et les preuves floues dans le PR au lieu de les laisser comme réviseurs
mémoire.

| Voie         | À utiliser lorsque le changement touche                                                                                                               | Preuves à enregistrer                                                                                                                                                                                                                                                                                  |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Sécurité     | Gestion des URL, sortie HTML ou SSR, chargement du système de fichiers/configuration, chargement natif, publication de paquets, CI ou identifiants.   | `security-audit` dans `.github/workflows/check.yml`, `vp exec pnpm audit --prod --audit-level moderate`, `cargo audit --deny warnings`, les vérifications d’installation fumigène, les vérifications d’actions GitHub épinglées, et toute régression ciblée couvrant l’entrée ou la frontière risquée. |
| Performances | Parser, compilateur, linter, formateur, vérificateur de type, mise en cache, parcours de graphes projet, sortie générée ou E/S CLI.                   | `.github/workflows/benchmark.yml`, `tools/benchmarks/scripts/compare-pr.mjs`, `tools/benchmarks/scripts/enforce-pr-budget.mjs`, le statut de `pr-benchmark-budget`, les tâches locales de `bench:*`, et `vize lint --profile`, `vize check --profile`, ou `vize fmt --profile` sortie lorsque la régression nécessite une attribution.       |
| Fuzzing      | Analyse syntaxique orientée octets, récupération de syntaxe, analyse CSS, analyse d’expressions JS/TS, lexing de modèles ou récupération de codegens. | `.github/workflows/fuzz.yml`, `tests/fuzz/Cargo.toml`, `tools/commands/ci/fuzz/seed_corpus.rs`, `cargo +nightly fuzz run <target>`, `fuzz-reproducers-*` téléchargé, et une régression déterministe minimisée après le plantage, le temps d’attente ou l’OOM a été compris.                                       |

## Politique de base

- Commencez par le plus petit cas défaillant ou illustratif, puis acceptez les installations plus larges uniquement lorsque celles-ci
  s’avère être un comportement transversal.
- Les fichiers snapshot et de référence sont des contrats visibles par l’utilisateur. Si un différent modifie les diagnostics, généré
  code, sortie CLI publique ou comportement de l’éditeur, le RP doit expliquer pourquoi la nouvelle sortie est correcte.
- Normaliser les données volatiles avant qu’elles n’atteignent une base de référence. Chemins, timings, hachages et environnement
  détails ne doivent pas créer un churn récurrent de snapshots.
- Gardez explicites les artefacts de parité. `tests/snapshots/check`, `tests/snapshots/lint`, le monde réel
  instantanés de luminaires et la matrice de parité Vue sont l’enregistrement de compatibilité.
- Ne rafraîchissez pas de grandes lignes de base snapshot à moins que la PR ne concerne ces sorties. Lorsque de nombreux fichiers bougent
  ensemble, incluez une brève explication de la cause commune.

## Déclencheurs d’escalade

Faites preuve plus large lorsqu’un changement prend l’une de ces formes :

- La syntaxe, la transformation ou le comportement virtuel de TypeScript pouvaient affecter les applications Vue ordinaires :
  ajouter ou mettre à jour un équipement réel et expliquer la parité avec les outils officiels de Vue.
- La forme de code générée, la mise en cache, la traversée de graphes de projet ou l’analyse sensible aux types pouvaient évoluer
  débit : effectuer le benchmark local qui correspond à la surface et compter sur le budget de référence PR.
- Gestion des URL, sortie HTML/SSR, chargement de configuration, publication de paquets, chargement natif, CI, ou
  modifications de code adjacentes aux identifiants : enregistrez le statut d’audit de sécurité et ajoutez la régression ciblée qui prouve
  que la frontière est toujours gardée.
- Récupération d’analyseurs, entrée d’octets arbitraire, analyse CSS/modèle/expression, ou corrections de plantage : run ou
  planifier la cible de fuzz correspondante, conserver le reproducteur et obtenir une régression déterministe
  minimisée avant de fermer la demande de correction.
- LSP, éditeur, correctif rapide, complétude, survol ou changements de comportement incrémentaux : ajouter au niveau du scénario
  une couverture qui exerce la séquence visible de l’utilisateur, pas seulement le diagnostic final.
- Un instantané change en raison des chemins, hachages, ordre, timing, environnement ou plateforme hôte :
  normaliser d’abord, puis n’accepter la référence que si le diff restant a un sens.

## Garde-corps opérationnels

Vize maintient ces pratiques exécutables au lieu de s’appuyer sur la mémoire :

- `CONTRIBUTING.md` nomme la discipline de changement de classe pour les contributeurs.
- `.github/PULL_REQUEST_TEMPLATE.md` demande des références comportementales, des risques et des preuves de vérification.
- `tools/benchmarks/scripts/test-inventory.mjs` rapporte l’inventaire actuel des actifs de test dans PR CI.
- `.github/workflows/benchmark.yml` compare la performance CLI de base et de la tête et applique un budget RP.
- `.github/workflows/check.yml` gère le `security-audit` pour le npm de production et Rust
  avis sur la dépendance.
- `.github/workflows/fuzz.yml` lance l’espace de travail cargo-fuzz `tests/fuzz` et les uploads plantent
  reproducteurs pour le triage analyseur/compilateur.
- `docs/release/production-readiness.md` et `docs/release/vue-parity-matrix.md` définissent quand un
  comportement peut être qualifié de prêt pour la production ou de compatible.
- `tests/tooling/language-engineering-practices.test.ts` conserve cette page, le guide des contributions,
  et le modèle de RP sont connectés.
