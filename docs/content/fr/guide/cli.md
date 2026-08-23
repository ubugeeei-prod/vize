---
title: CLI
---

<!-- Generated translation; source: guide/cli.md -->

# Référence CLI

> **⚠️ Travaux en cours :** Vize est en développement actif et la surface du CLI est encore en évolution.

La plupart des flux de travail applicatifs devraient installer le paquet `vize` npm et le faire passer à travers `package.json`
scripts. Cette page décrit le `vize` binaire Rust-native de bas niveau pour LSP, gestion IDE,
`check-server`, profilage et autres flux de travail directs de CLI. Le package npm expose des assistants de configuration
partagés ainsi que des commandes `build`, `fmt`, `lint`, `check`, `clean`, `ready`et `upgrade` soutenues par NAPI.

Pour une explication plus large du pipeline d’analyse, voir [Static Analysis](./static-analysis.md).

## Scripts de paquets d’application

Pour les applications, installez depuis npm et connectez les commandes stables dans les scripts de projet :

```bash
vp install -D vize
```

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Utilisez `vp exec vize ...` pour un débogage local ponctuel, mais privilégiez les scripts nommés pour les flux de travail
documentés et les CI.

## Installation binaire de rouille

Pour l’alpha v1, utilisez les binaires de version GitHub préconstruits ou le point d’entrée Nix. Le CLI Rust n’est pas encore un canal
crates.io pris en charge.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

Vous pouvez également télécharger des binaires spécifiques à la plateforme depuis
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases).

Pour le développement local dans ce dépôt, installez la construction workspace :

```bash
cargo install --path crates/vize --force --locked
```

## Scripts de paquets npm vs Rust CLI

| Besoin                                                                              | Point d’entrée recommandé     |
| ----------------------------------------------------------------------------------- | ----------------------------- |
| Packer des scripts pour build, format, lint, check, ready et upgrade                | `vp run vize:*` du paquet NPM |
| Vérification de type soutenue par projet sur `.vue`, `.ts`, `.tsx`et `.d.ts`        | Rouille `vize check`          |
| LSP, configuration de l’IDE, `check-server`et artefacts de profilage                | Rouille `vize` binaire        |
| Plugin Shared Vite, commande package npm et paramètres de la ligne de commande Rust | `vize.config.*`               |

## Commandements

```bash
vize [COMMAND]
```

Lorsqu’il est invoqué sans commande, `vize` passe par défaut à `build`.

| Commandement   | Description                                                             |
| -------------- | ----------------------------------------------------------------------- |
| `build`        | Compiler les fichiers SFC de Vue                                        |
| `fmt`          | Formatez les fichiers SFC de Vue                                        |
| `lint`         | Fichiers SFC Lint Vue                                                   |
| `check`        | Entrées de contrôle de type Vue SFC, TS, TSX et `.d.ts`                 |
| `inspector`    | Créer des charges utiles pour l’inspecteur du compilateur de playground |
| `clean`        | Supprimer les artefacts de cache générés par Vize                       |
| `ready`        | Faites `fmt`, `lint`, `check`et `build`                                 |
| `upgrade`      | Mettre à jour la ligne de commande installée                            |
| `check-server` | Démarrez le serveur Unix JSON-RPC de vérification de type               |
| `musea`        | Sous-commandements et échafaudages de Musea                             |
| `lsp`          | Démarrez le serveur de langue                                           |
| `ide`          | Installer ou gérer les intégrations de l’éditeur                        |

Tous les rapports `--profile` terminaux sont effectués par la caisse `vize_curator` locale uniquement. Les
crochets d’instrumentation restent en `vize_carton`, tandis que le conservateur possède la forme du rapport CLI aux côtés
artefacts faisant face à l’inspecteur et à l’agent.

## Build

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

Options clés :

| Option                | Description                                                                                   |
| --------------------- | --------------------------------------------------------------------------------------------- |
| `-o, --output`        | Sortie relative à la source en dessous de la racine d’entrée commune ; rejette les collisions |
| `-f, --format`        | Format de sortie : `js`, `json`, `stats`                                                      |
| `--ssr`               | Activer la compilation SSR                                                                    |
| `--custom-renderer`   | Considérer les balises minuscules non HTML comme des éléments de rendu personnalisés          |
| `--custom-elements`   | Motifs de balises compilés comme éléments personnalisés ; répétable                           |
| `--script-ext`        | `preserve` ou `downcompile`                                                                   |
| `--declaration`       | Émettre `.d.ts` fichiers pour les SFC construits (alias : `--dts`)                            |
| `--declaration-dir`   | Dossier de sortie de déclaration (par défaut : le répertoire de sortie de compilation)        |
| `-j, --threads`       | Remplacement du nombre de fils                                                                |
| `--profile`           | Profil de timing d’impression                                                                 |
| `--continue-on-error` | Continuez à compiler et signalez les échecs à la fin                                          |

## Format

```bash
vize fmt --check src
vize fmt --write src
```

Options clés :

| Option                             | Description                                        |
| ---------------------------------- | -------------------------------------------------- |
| `--check`                          | Fichiers de rapports qui allaient changer          |
| `-w, --write`                      | Écriture de sortie formatée                        |
| `--single-quote`                   | Style de guillemets à bascule de chaîne            |
| `--print-width`                    | Largeur maximale de ligne                          |
| `--tab-width`                      | Largeur d’indentation                              |
| `--use-tabs`                       | Bascule des onglets vs espaces                     |
| `--no-semi`                        | Omettez les points-virgules                        |
| `--sort-attributes`                | Attributs du modèle de tri                         |
| `--single-attribute-per-line`      | Mettez un attribut par ligne                       |
| `--max-attributes-per-line`        | Enrouler après un certain nombre d’attributs       |
| `--normalize-directive-shorthands` | Normaliser `v-bind:` / `v-on:` / `v-slot:` abrégés |
| `--profile`                        | Profil de timing d’impression                      |

## Peluches

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

Options clés :

| Option                | Description                                                                                                                                    |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `--fix`               | Appliquez des correctifs automatiques sûrs issus des règles qui fournissent des modifications de texte, puis signalez les diagnostics restants |
| `-f, --format`        | Format de sortie : `text`, `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`ou `agent`                                                    |
| `--max-warnings`      | Échec lorsque les avertissements dépassent la limite                                                                                           |
| `-q, --quiet`         | Résumé de l’émission uniquement                                                                                                                |
| `--help-level`        | `full`, `short`, ou `none`                                                                                                                     |
| `--preset`            | `happy-path`, `opinionated`, `essential`, `incremental`ou `nuxt`                                                                               |
| `--cross-file`        | Activez les vérifications croisées par option                                                                                                  |
| `--cross-file-tree`   | Imprimez l’arbre de fournisseur/injection lorsque le linting entre fichiers est activé                                                         |
| `--strict-reactivity` | Activer le linting de perte de réactivité supporté par un checker natif                                                                        |
| `--profile`           | Profil de timing d’impression                                                                                                                  |
| `--slow-threshold`    | Seuil lent de fichier pour la sortie de profil                                                                                                 |

Les préréglages sont destinés à une adoption progressive :

| Préréglage    | Utilisez-le quand                                                                                |
| ------------- | ------------------------------------------------------------------------------------------------ |
| `essential`   | Vous voulez des diagnostics orientés vers la correction en IC                                    |
| `happy-path`  | Vous voulez le pack recommandé par défaut                                                        |
| `opinionated` | Vous voulez des conventions plus fortes, des règles de script et des candidats sensibles au type |
| `incremental` | Vous ne voulez que des règles explicitement configurées                                          |
| `nuxt`        | Vous voulez des règles opiniâtres avec des hypothèses de composants Nuxt                         |

Exemples :

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format ansi src
vize lint --format plain src
vize lint --format agent src
vize lint --format markdown src
```

## Vérifié

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` est soutenu par `vize_canon` et des sessions de projets Corsa exposées via [`corsa-bind`](https://github.com/ubugeeei/corsa-bind). Vize génère un TypeScript virtuel pour les SFC Vue, exécute des diagnostics de projet sur un chemin natif, et mappe les résultats vers les emplacements sources d’origine.

Lorsque aucun chemin explicite n’est donné, `vize check` utilise `tsconfig.json` `files` / `include` /
`exclude` si disponible. Les entrées explicites peuvent être des fichiers, des répertoires ou des globules et peuvent inclure `.vue`,
`.ts`, `.tsx`et `.d.ts`.

Options clés :

| Option              | Description                                                    |
| ------------------- | -------------------------------------------------------------- |
| `-s, --socket`      | Connectez-vous à un `check-server`                             |
| `--tsconfig`        | Outrepose `tsconfig.json`                                      |
| `-f, --format`      | Format de sortie : `text` ou `json`                            |
| `--show-virtual-ts` | TypeScript virtuel généré par l’impression                     |
| `-q, --quiet`       | Résumé de l’émission uniquement                                |
| `--profile`         | Écrivez les artefacts de profil sous `node_modules/.vize`      |
| `--corsa-path`      | Écraser le chemin exécutable Corsa                             |
| `--servers`         | Nombre réservé de serveurs Corsa ; Seul `1` est pris en charge |
| `--declaration`     | Émettre `.d.ts` sortie                                         |
| `--declaration-dir` | Répertoire de sortie pour les déclarations émises              |

Utilisez-`--corsa-path` lorsque vous souhaitez épingler un exécutable Corsa personnalisé lors du développement de Vize ou en testant un
`corsa-bind` local de vérification. La clé de configuration partagée est `typeChecker.corsaPath`; `typeChecker.tsgoPath`
est conservé uniquement comme alias de compatibilité.

Motifs utiles :

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

Les valeurs de gabarit à l’échelle du projet et les types d’ambiance Vue doivent être visibles via la configuration
projet TypeScript. Incluez des fichiers générés tels que `auto-imports.d.ts`, `components.d.ts`ou vos propres déclarations
Vue dans `tsconfig.json`, puis sélectionnez ce projet avec `--tsconfig` lorsque nécessaire :

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
```

## Inspecteur

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector` emballe un ou plusieurs fichiers `.vue` dans la charge utile consommée par le playground
l’inspecteur du compilateur. Le navigateur inspecte ensuite la sortie Vue, la sortie Vize, Virtual TS, VIR, ainsi que le graphique
croisé, puis produit un permalien ainsi qu’un lien pull request prérempli.

Utilisez `--format agent` quand un autre outil local ou un agent IA a besoin du même repro, sans ouvrir le navigateur
. Le rapport contient la charge utile exacte, l’URL du playground, les métriques de résumé et le graphique d’importation.
Les métadonnées de charge utile, graphe et différentiel de ligne sont construites par la caisse de `vize_curator` locale uniquement afin que
inspection CLI et de terrain de jeu restent alignées.

Options clés :

| Option              | Description                                               |
| ------------------- | --------------------------------------------------------- |
| `-f, --format`      | Format de sortie : `url`, `json`ou `agent`                |
| `--target`          | Cible du compilateur : `dom` ou `ssr`                     |
| `--playground-url`  | URL de base de Playground pour les liens générés          |
| `--max-files`       | Fichiers de limite inclus dans une charge utile batch     |
| `--custom-renderer` | Activer la comparaison des moteurs de rendu personnalisés |
| `--template-syntax` | Choisissez `standard`, `strict`ou `quirks`                |
| `-o, --output`      | Écrire l’URL ou la charge utile JSON dans un fichier      |

Voir [Compiler Inspector](./compiler-inspector.md) pour le flux de travail des contributeurs.

## Clean

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` supprime les artefacts locaux connus appartenant à Vize pour la racine du projet sélectionné, puis supprime
`.vize` vides et `node_modules/.vize` parents. La liste d’artefacts gérés couvre les sorties de profil,
rapports/snapshots/tokens Musea, sessions Patina, schémas de configuration, journaux LSP, restes de sockets, dumps de
OXC, fichiers de contournement Oxlint et fichiers de projets Corsa matérialisés. Les entrées inconnues sous `.vize`
sont préservées par défaut ; Utilisez-`--force` uniquement lorsque la racine de l’artefact sélectionnée doit être retirée
en gros. `--dry-run` imprime les chemins d’artefacts qui seraient supprimés. Utilisez `--scope node-modules`
ou `--scope project` lorsque seule une racine d’artefact doit être nettoyée.

## Prêt

```bash
vize ready src
vize ready --output dist src
```

`vize ready` s’exécute `fmt --write`, `lint`, `check`et `build` dans l’ordre. La commande s’arrête à la
première étape ratée.

Options clés :

| Option         | Description                                      |
| -------------- | ------------------------------------------------ |
| `-o, --output` | Répertoire de sortie pour l’étape de compilation |
| `--ssr`        | Activer la compilation SSR pour la compilation   |
| `--script-ext` | `preserve` ou `downcompile`                      |

## Mise à niveau

```bash
vize upgrade
vize upgrade --dry-run
```

Par défaut, `vize upgrade` met à jour le package npm via Vite+ :

```bash
vp install -D vize@latest
```

Utilisez `--source cargo` uniquement pour des installations locales explicites de Cargo.

## Musea

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

Le sous-commandement `musea` se concentre actuellement sur l’échafaudage et les points d’entrée expérimentaux.
Pour le développement quotidien de galeries, le flux de travail recommandé aujourd’hui est
`@vizejs/vite-plugin-musea`.

Le package npm expose également une commande `vize musea` pratique qui exécute Vite avec le plugin Musea
installé dans votre projet :

```bash
vp exec vize musea
vp exec vize musea --build
```

## LSP et IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp` lance directement le serveur de langue.
`vize ide` ajoute des commandes d’installation et de gestion spécifiques à l’éditeur pour les intégrations VS Code et Zed
.

## Options mondiales

```bash
vize --help
vize --version
vize <command> --help
```
