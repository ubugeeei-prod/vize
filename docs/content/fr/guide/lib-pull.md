---
title: Distribution des sources (vize lib)
---

<!-- Generated translation; source: guide/lib-pull.md -->

# Distribution des sources (`vize lib`)

`vize lib` copie les composants de `@vizejs/ui` et les composables de `@vizejs/composable` dans votre
projet **sous forme de sources**, à la manière de shadcn/ui. Les fichiers copiés vous appartiennent : modifiez-les
librement, et Vize enregistre la version de paquet d'où provient chaque fichier pour que les mises à jour restent sûres.

```bash
vpx vize lib pull rating
```

Cela écrit la famille `rating` ainsi que tout ce qu'elle importe (`controllable-state`, `id`, ...) dans
`src/components/vize/`, et consigne l'opération dans `vize-lib.lock.json`.

## Provenance des sources

Chaque tarball publié de `@vizejs/ui` et `@vizejs/composable` contient un registre versionné :

```text
node_modules/@vizejs/ui/
  registry/
    registry.json          # éléments, fichiers, empreintes sha256, dépendances
    files/families/...     # sources .vue / .ts / .css brutes (tests exclus)
```

`vize lib` résout un registre dans cet ordre :

1. `--registry <path>` : un `registry.json`, son répertoire ou un répertoire de paquet décompressé. Répétez
   l'option pour fournir les registres ui et composable. La découverte automatique est alors désactivée.
2. Le paquet installé dans le projet (`node_modules/@vizejs/ui/registry/registry.json`, recherché dans les
   répertoires parents comme la résolution Node).
3. `npm pack @vizejs/<pkg>@<version>` dans un répertoire temporaire suivi de `tar -xzf`, lorsque la demande fixe
   une version différente de celle installée ou que le paquet n'est pas installé (alors `@latest`). Utilisez
   `--offline` pour interdire cette étape.

Aucun serveur de registre ni client HTTP supplémentaire n'est utilisé : le registre est exactement le tarball que
npm sert déjà, donc chaque version est immuable et reproductible.

## Commandes

| Commande                                 | Rôle                                                                                    |
| ---------------------------------------- | --------------------------------------------------------------------------------------- |
| `vize lib init [--dry-run]`              | Détecte la structure du projet et écrit la section de configuration `lib`.              |
| `vize lib list [--kind ui\|composable]`  | Liste les éléments disponibles.                                                         |
| `vize lib search <words>`                | Recherche par nom, titre, description et alias.                                         |
| `vize lib info <name>`                   | Affiche les fichiers, dépendances de registre, peers npm et la version du paquet.       |
| `vize lib pull <item>... [--dir <dir>]`  | Copie des éléments et leurs dépendances de registre ; `--dry-run`, `--overwrite`.       |
| `vize lib add <item>...`                 | Alias de `pull` compatible avec shadcn (`-p/--path`, `-o/--overwrite`, `-y/--yes`).     |
| `vize lib status`                        | Compare les fichiers copiés au lockfile et au registre installé.                        |
| `vize lib diff <name> [--to <version>]`  | Diff unifié de votre copie locale vers une version du registre.                         |
| `vize lib update [<name>...] [--to <v>]` | Applique les changements amont sans écraser vos modifications ; `--dry-run`, `--force`. |
| `vize lib remove <name>...`              | Supprime des éléments et les dépendances devenues inutiles ; `--dry-run`, `--force`.    |
| `vize lib outdated`                      | Compare les versions verrouillées aux registres installé et le plus récent.             |

Toutes les commandes acceptent `--json` pour une sortie lisible par machine et `--root <dir>` pour cibler un autre projet.

### Désigner les éléments

Les éléments se désignent par leur nom canonique (`rating`), par alias (`star rating`, `useToggle`), avec un
préfixe de type lorsqu'un nom existe dans les deux paquets (`ui:locale`, `composable:locale`), et avec une version
exacte du paquet (`rating@0.427.0`, `composable:use-toggle@0.427.0`).

### Répertoires cibles

Les fichiers copiés conservent la disposition du registre (`families/form/rating/rating.vue`,
`foundations/id/deterministic-id.ts`, ...) sous un répertoire par type, de sorte que les imports relatifs entre
éléments continuent de fonctionner sans réécriture. Le répertoire est choisi par :

1. `--dir <dir>` (doit rester dans le projet),
2. la section `lib` de `vize.config.*`,
3. la valeur par défaut du registre : `src/components/vize` (ui) et `src/composables/vize` (composable).

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  lib: {
    uiDir: "src/ui/vendor",
    composableDir: "src/composables/vendor",
    // dir: "src/vendor",        // repli commun aux deux types
    // lockfile: "vize-lib.lock.json",
  },
});
```

Une fois un type copié dans un répertoire, les copies suivantes de ce type le réutilisent ; un `--dir`
contradictoire est refusé plutôt que de scinder le graphe de dépendances.

Les sources copiées n'importent que des chemins relatifs et des paquets npm comme `vue`. `pull` signale toute
dépendance npm que votre `package.json` ne déclare pas encore ; il n'installe jamais de paquet à votre place.

## Premiers pas : `init`

```bash
vize lib init --dry-run   # affiche la structure détectée et la modification de configuration
vize lib init             # l'écrit
```

`init` détecte le répertoire des sources (`src/`, ou `app/` pour les projets Nuxt 4) et TypeScript, puis écrit
`lib.uiDir` / `lib.composableDir`. Il crée `vize.config.json` en l'absence de configuration, ajoute une section
`lib` à un `vize.config.json` existant sans toucher au reste du fichier, et affiche un extrait pour
`vize.config.ts` / `.pkl` au lieu de modifier du code. Une section `lib` existante est conservée sauf avec
`--force`. Si `tsconfig.json` n'active pas `allowImportingTsExtensions`, `init` le signale : les sources copiées
importent leurs voisins sous la forme `./x.ts`.

## Vérifier les mises à jour : `outdated`

`vize lib outdated` liste chaque élément copié dont le registre diffère du lockfile :

| Colonne   | Signification                                                                         |
| --------- | ------------------------------------------------------------------------------------- |
| `current` | Version enregistrée dans `vize-lib.lock.json`.                                        |
| `wanted`  | Version du registre qu'utiliserait `update` (paquet installé, sinon le plus récent).  |
| `latest`  | Dernière version publiée sur npm (`npm view` ; ignorée avec `--offline`).             |
| `state`   | `update-available`, `newer-release`, `removed-upstream`, `unknown` (ou `up-to-date`). |

`update-available` signifie que le `contentHash` de l'élément a changé ; une nouvelle version qui ne touche pas
ses fichiers le laisse `up-to-date`. `--json` inclut tous les éléments.

## Registres tiers

N'importe quel paquet ou site peut publier un registre au même format et l'exposer sous un espace de noms :

```json
{
  "lib": {
    "registries": {
      "@acme": "npm:@acme/vue-kit",
      "@design": { "source": "https://design.example.com/r/registry.json", "dir": "src/design" },
      "@local": { "source": "./registry", "dir": "src/local" }
    }
  }
}
```

```bash
vize lib pull @acme/data-table @design/button@2.1.0
vize lib list --kind @acme
```

- `npm:<package>[@range]` utilise le `registry/registry.json` du paquet installé, sinon `npm pack`.
- `https://…/registry.json` est récupéré avec `curl`, et chaque fichier est téléchargé à la demande depuis `files/<path>` à côté (https uniquement).
- Toute autre valeur est un chemin relatif au fichier de configuration : un `registry.json`, son répertoire ou un répertoire de paquet.

Les registres tiers sont validés avec le même JSON Schema que les registres officiels (champs inconnus, empreintes
malformées, rôles ou dépendances inconnus et fermeture de dépendances incomplète sont refusés), et chaque octet
téléchargé est vérifié par SHA-256 avant d'être écrit. Les éléments d'un espace de noms vont dans son `dir` (sinon
le `defaultTargetDirectory` du registre), sont verrouillés sous la clé `@namespace` et ne peuvent jamais écraser un
fichier appartenant à un autre élément copié.

## Versionnement et mises à jour sûres

`vize-lib.lock.json` (à committer) enregistre, pour chaque élément, le paquet et la version exacte d'origine, le
`contentHash` du registre, s'il a été demandé explicitement ou ajouté comme dépendance, et le SHA-256 de chaque
fichier au moment de la copie. Ces empreintes servent de base de fusion à une comparaison à trois voies entre
**votre fichier**, **le fichier tel que copié** et **le fichier du nouveau registre** :

| Votre fichier vs copie | Registre vs copie | `update` / `pull`                                  |
| ---------------------- | ----------------- | -------------------------------------------------- |
| inchangé               | inchangé          | rien (`unchanged`)                                 |
| inchangé               | modifié           | le remplace (`update`)                             |
| modifié                | inchangé          | conserve votre modification (`keep-local`)         |
| modifié                | modifié           | refuse (`conflict`) sans `--force` / `--overwrite` |
| inchangé               | supprimé en amont | le supprime (`delete`)                             |
| modifié                | supprimé en amont | refuse (`conflict-delete`) sans `--force`          |
| absent                 | quelconque        | le restaure (`create`)                             |
| présent, hors lockfile | quelconque        | refuse (`conflict`) sans `--overwrite`             |

Rien n'est écrit tant qu'un conflit n'est pas résolu : une mise à jour refusée laisse fichiers et lockfile
intacts. Utilisez `vize lib diff <name> --to <version>` pour examiner le changement amont, fusionnez-le à la main,
puis lancez `update --force`.

Une mise à jour typique :

```bash
pnpm add @vizejs/ui@latest       # ou : vize lib update --to 0.428.0
vize lib status                  # éléments avec mises à jour / modifications locales
vize lib update --dry-run        # aperçu des actions sur les fichiers
vize lib update                  # applique ; les conflits éventuels sont listés
```

`remove` supprime un élément et toutes les dépendances copiées uniquement pour lui. Il refuse tant qu'un autre
élément copié importe encore la cible, et conserve les fichiers modifiés localement sauf avec `--force`.

## Format du registre

Le document de registre est décrit par
[`vize-lib-registry.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-registry.schema.json)
et le lockfile par
[`vize-lib-lock.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-lock.schema.json).
Les deux sont publiés dans le paquet npm `vize` sous `schemas/`.

- `registryDependencies` est la fermeture transitive complète calculée à partir du graphe d'imports relatifs
  lors du build ; les fondations partagées sont des éléments distincts, donc copier deux composants qui ont besoin
  de `id` ne copie `id` qu'une fois.
- Chaque fichier porte son `sha256` ; `vize lib` vérifie chaque octet copié.
- `contentHash` ne change que lorsque les fichiers d'un élément changent, donc `status` ne signale « update
  available » que pour les éléments dont les sources diffèrent réellement d'une version à l'autre.
