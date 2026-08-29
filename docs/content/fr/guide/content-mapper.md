---
title: TypeScript Content Mapper
---

<!-- Generated translation; source: guide/content-mapper.md -->

# TypeScript Content Mapper

Les Content Mappers sont la surface de plugins de TypeScript pour vérifier les types de fichiers
que le compilateur ne peut pas analyser lui-même — la
[feuille de route de l'API TypeScript 7.1](https://github.com/microsoft/typescript-go/issues/4830)
les identifie comme le remplaçant des plugins TS Server nécessaire à Vue. L'API a été fusionnée
dans la branche main de `typescript-go` via
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712).

Vize embarque un content mapper conforme dans le paquet npm `vize` : un build de `tsgo` prenant en
charge les content mappers lance `vize content-mapper` et vérifie directement les fichiers `.vue` —
survol, aller à la définition, renommage, complétions et diagnostics sont tous reprojetés vers
votre SFC d'origine, sans matérialiser de projet `.vue.ts` parallèle.

> **⚠️ Aperçu :** Les Content Mappers sont fusionnés upstream mais pas encore présents dans les
> packages TypeScript 7 platform publiés. Tant qu'une version n'inclut pas le protocole, compilez
> un binaire TypeScript natif avec Content Mapper depuis la main de `typescript-go` et gardez
> [`vize check`](./cli.md#check) comme chemin de vérification de types pris en charge.

## Configuration

Installez `vize` et déclarez le mapper dans votre `tsconfig.json` :

```bash
vp install -D vize
```

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true
  },
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"]
    }
  ],
  "include": ["src"]
}
```

L'exécution de processus mappers externes exige un opt-in explicite :

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

Dans VS Code, l'extension Vize enregistre automatiquement la prise en charge de `.vue` auprès de
l'hôte content-mapper TypeScript 7 dans les espaces de travail approuvés — le même mapper alimente
alors l'éditeur.

## Options

Une entrée de mapper accepte un objet `options` :

```json
{
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"],
      "options": { "optionsApi": false }
    }
  ]
}
```

| Option       | Défaut | Rôle                                                                   |
| ------------ | ------ | ----------------------------------------------------------------------- |
| `optionsApi` | `true` | Résoudre les liaisons d'instance de l'Options API Vue dans les templates |

Des options invalides ne font jamais échouer le build : Vize les signale comme diagnostics
d'option positionnés dans votre tsconfig (`vize1`–`vize3`) et continue avec les valeurs par
défaut. Vize déclare aussi une dépendance à l'option de compilation `noUnusedLocals` du projet, si
bien que le signalement des variables locales inutilisées dans `<script setup>` suit la
configuration de chaque projet.

## Directives de Template

`@ts-expect-error` fonctionne normalement dans les blocs `<script>`, qui passent tels quels. Les
expressions de template ne peuvent pas porter de commentaires TS, donc Vize projette les
directives de commentaire HTML standard de Vue à travers le protocole :

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` supprime les diagnostics TypeScript sur la ligne de template
  suivante et signale `vize4: Unused '@vue-expect-error' directive` quand rien n'a été supprimé.
- `<!-- @vue-ignore -->` supprime silencieusement.

Une directive s'applique au reste de sa propre ligne quand du contenu suit le commentaire, sinon à
la ligne non vide suivante.

## Protocole

Vize parle le protocole v1 des content mappers tel que fusionné upstream : encodage de positions
UTF-8, cycle de vie `openProject`/`closeProject` par projet, et sortie virtuelle `.tsx` pour que
TypeScript et le JSX embarqué soient analysés correctement. La conformité est garantie en CI
contre une révision épinglée de `typescript-go`, qui compile le compilateur upstream exact et
exécute les suites complètes CLI, build et LSP à travers les artefacts npm empaquetés.

Codes de diagnostic signalés sous la source `vize` :

| Code    | Signification                                       |
| ------- | ---------------------------------------------------- |
| `vize1` | La valeur des options du mapper n'est pas un objet   |
| `vize2` | Option de mapper inconnue                            |
| `vize3` | Option de mapper de type incorrect                   |
| `vize4` | Directive `@vue-expect-error` inutilisée             |

## Limitations

- Nécessite un `tsgo` compilé depuis la main de `typescript-go` tant qu'une version de TypeScript 7
  n'inclut pas l'API.
- Les declaration maps pour les entrées mappées attendent
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860).
- `vize check` reste le chemin de vérification de types pris en charge en production tant que
  l'API upstream est en aperçu.
