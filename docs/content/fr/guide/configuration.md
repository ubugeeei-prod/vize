---
title: Configuration
description: Partagez les réglages Vize depuis votre configuration Vite et votre projet TypeScript.
---

<!-- Reviewed translation; source: guide/configuration.md -->

# Configuration

Gardez les réglages Vize dans `vite.config.ts` et le projet TypeScript dans `tsconfig.json`. Les valeurs par défaut ne nécessitent aucun fichier Vize supplémentaire.

> [!NOTE]
> La découverte de la configuration Vite par la CLI et l'éditeur natifs, ainsi qu'init sans fichier dédié, sont en préparation pour la prochaine version. D'ici là, les outils natifs publiés utilisent encore le format dédié existant pour les réglages partagés personnalisés. La [référence](./configuration-reference.md) décrit ce format.

## Configuration Vite+

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

| Emplacement | Fonction | Commande |
| --- | --- | --- |
| `compiler` | Compilation Vue | `vp dev`, `vp build` |
| `lint.vize` | Règles lint Vue | `vp run lint` |
| `fmt.vize` | Formatage Vue | `vp run fmt:check` |
| `typecheck` | Vérification des types Vue | `vp run typecheck` |
| `pack.vize` | Déclarations de bibliothèque | `vp run pack` |

Utilisez `vp run check` pour les tâches Vize combinées. Les commandes intégrées `vp check`, `vp lint` et `vp fmt` conservent le fonctionnement propre à Vite+. Un script existant peut renommer une tâche générée en `vize:<nom>` ; consultez les [noms et remplacements des tâches](./vite-plus.md#tasks).

## Modifier une règle

Placez les règles Vue dans `lint.vize.rules` et celles d'Oxlint dans `lint.rules`. Les [options des règles](../rules/options.md) décrivent les paramètres et le [catalogue](../rules/all.md) donne les exemples complets.

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

## Choisir les fonctions à adopter

Définissez `compiler`, `typecheck`, `lint.vize` ou `fmt.vize` sur `false` pour désactiver la fonction concernée. Vize formate Vue et Oxfmt les autres fichiers. Consultez la [répartition et la gestion des conflits](./vite-plus.md#lint-and-formatter-ownership).

## Vite classique

Utilisez le plugin pour la compilation et un objet `vize` à la racine de la configuration pour les réglages partagés avec la CLI et l'éditeur. L'import du plugin ajoute aussi les types de configuration Vite.

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vize: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
    typeChecker: { strict: true },
  },
});
```

<span id="standalone-cli"></span>

## CLI autonome

Installez `vize` et lancez les commandes depuis la racine du paquet ciblé. La CLI lit aussi `vite.config.*` et votre projet TypeScript. Les réglages Vite+ `compiler`, `typecheck`, `lint.vize` et `fmt.vize` sont également traduits pour les commandes natives. Les réglages natifs explicites de l'objet `vize` ont priorité sur cette traduction.

```bash
vp install -D vize
vp exec vize check
```

La recherche s'arrête au répertoire contenant le plus proche `package.json`, `tsconfig.json` ou `jsconfig.json`. Dans un monorepo, lancez les commandes depuis le paquet ciblé ou utilisez `vize.entries`, et choisissez explicitement les dossiers de travail de l'éditeur. La découverte des configurations Vite imbriquées par document et la prise en charge équivalente de `vite.root` par la CLI autonome sont encore en cours.

## Configuration dédiée facultative

Un fichier `vize.config.*` existant reste accepté et a priorité sur la configuration Vite du même répertoire. L'option CLI `--config` sélectionne un fichier explicitement. Les options directes du plugin et les fonctions explicitement choisies dans l'éditeur ont priorité sur les réglages partagés ; `config: false` désactive le chargement automatique par le plugin.

<span id="fichiers-de-configuration"></span>
<span id="configuration-typescript"></span>
<span id="résolution-de-type-vue"></span>
<span id="entrées-expérimentales-sur-le-plat"></span>
<span id="configuration-pkl"></span>
<span id="configuration-json"></span>
<span id="options-du-compilateur"></span>
<span id="syntaxe-des-modèles"></span>
<span id="mode-de-sortie-jsx-tsx"></span>
<span id="directives-par-composant"></span>
<span id="préséance"></span>
<span id="diagnostic"></span>
<span id="dialecte-vue"></span>
<span id="options-d-analyse-statique"></span>
<span id="musea-options"></span>

## Référence détaillée

La [référence partagée](./configuration-reference.md) décrit la recherche des fichiers, les formats dédiés TypeScript/JSON/PKL, les entrées par périmètre, la résolution des types Vue et les réglages LSP/Musea. Consultez la [référence du compilateur](./compiler-configuration-reference.md) pour ses options et ses syntaxes, et les [fonctions expérimentales](./experimentals.md) pour les options à activer explicitement.

