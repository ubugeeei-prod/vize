---
title: Flux de travail utilisateurs
---

<!-- Generated translation; source: guide/workflows.md -->

# Flux de travail utilisateurs

Ce guide propose un chemin compact à travers les flux de travail courants de Vize : l’installer, connecter la config,
formater, lint, vérifier le type, compiler et exécuter les mêmes portes dans CI.

## Installation

Installez le package npm dans le projet qui possède vos dépendances Vue :

```bash
vp install -D vize
```

Pour les monodépôs, installez-le à la racine de l’espace de travail lorsque les packages partagent un seul fichier de verrouillage. Installez-le dans un paquet
uniquement lorsque ce paquet a son propre fichier de verrouillage et graphe de dépendances.

## Ajouter des scripts de package

Privilégiez les scripts nommés aux commandes ponctuelles afin que les exécutions locales et CI partagent les mêmes points d’entrée :

```json
{
  "scripts": {
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
    "vize:check": "vize check src",
    "vize:build": "vize build src",
    "vize:ready": "vize ready src"
  }
}
```

`vize ready` est la large porte locale. Dans les dépôts plus grands, conservez aussi les commandes individuelles afin
développeurs puissent isoler la mise en forme, le lint, la vérification de type et les défaillances du compilateur.

## Configurer une fois

Créez `vize.config.ts` à la racine du projet lorsque les valeurs par défaut ne suffisent pas :

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

Voir [Configuration](./configuration.md) pour les entrées monorepo plates, PKL, JSON, les options du compilateur et
détails sur la résolution des types Vue.

## Format

Utilisez le mode vérification en CI et le mode écriture localement :

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

Pour des missions de migration ponctuelles, `vize fmt --write` peut cibler un fichier, un répertoire ou un glob.

## Peluches

Commencez par `happy-path` pour la correction et les diagnostics Vue à faible bruit :

```bash
vize lint --preset happy-path --max-warnings 0 src
```

Utilisez `--help-level short` quand la sortie CI doit rester compacte, et `--format json` quand un autre outil
consommera le diagnostic. Voir [CLI](./cli.md) et [Rules](../rules/index.md) pour la règle complète
surface.

## Vérification du type

Exécutez `vize check` depuis la racine du projet afin que les `tsconfig`actives, la version de Vue, les paquets du framework, les
et les types d’ambiance proviennent du même graphe de dépendances :

```bash
vize check src
```

Pour les vérifications monodépôt spécifiques à chaque package, exécutez depuis le répertoire package ou définissez `typeChecker.tsconfig`
dans une entrée de configuration à portée de contrôle.

## Compiler

Utilisez `vize build` lorsque vous avez besoin d’une sortie du compilateur en dehors du chemin du plugin Vite :

```bash
vize build src --output dist/vize
```

Pour les applications Vite, privilégiez `@vizejs/vite-plugin` et laissez Vite gérer l’orchestration de build. Voir
[Vite Plugin](./vite-plugin.md).

## CI

Utilisez les mêmes scripts de package dans CI :

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

Gardez `vize:build` dans la porte uniquement lorsque le projet consomme directement la sortie du compilateur Vize. Pour
applications Vite, la compilation normale de l’application exerce le plugin.

## Débogues

Lorsqu’une défaillance n’est pas claire :

- relancer avec `--format json` pour inspecter des champs de diagnostic stables ;
- utilisez `--profile` sur `check`, `lint`ou `build` pour trouver des phases lentes ;
- créer une charge utile d’inspecteur avec `vize inspector` pour les incompatibilités du compilateur ;
- Incluez la plus petite tranche de fichier `.vue` ou de projet lors de la demande d’une correction.

Les pages [Testing & Feedback](./testing.md) et [Troubleshooting](./troubleshooting.md) couvrent les reportages
, les événements réels et les problèmes environnementaux courants.
