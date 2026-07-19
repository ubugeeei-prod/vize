---
title: Musea
---

<!-- Generated translation; source: guide/musea.md -->

# Musea

> **⚠️ Travaux en cours :** La musea est encore en évolution. Les formats de fichiers, les API et le comportement de l’interface utilisateur peuvent changer.

Musea est la chaîne d’outils de fichiers d’art et de galerie de composants de Vize.

- `vize_musea` 'est le noyau Rust pour analyser `*.art.vue`, générer des documents, construire des palettes d’accessoires,
  génération automatique de variantes et préparation des données VRT.
- `@vizejs/vite-plugin-musea` est la galerie recommandée et le flux de travail dev-server aujourd’hui.
- `musea-vrt` 'est la CLI pour les instantanés de régression visuelle, les audits a11y, les approbations, le nettoyage, et
  générait des fichiers d’art.

## Aperçu

![Musea Component Gallery — Home](/musea-home.png)

Musea utilise `*.art.vue` fichiers pour décrire des variantes de composants avec une syntaxe native Vue.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez le package :

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Utilisation recommandée : Plugin Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

Exécutez votre serveur de développement Vite habituel et ouvrez la route Musea configurée :

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

Si vous installez le package `vize` npm, `vp exec vize musea` est un wrapper pratique autour de Vite :

```bash
vp exec vize musea
vp exec vize musea --build
```

## Configuration partagée

`musea()` options suppriment la configuration partagée. Mettez les paramètres par défaut stables du projet dans `vize.config.ts` et gardez
paramètres de prévisualisation uniquement dans `vite.config.ts`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

La configuration partagée couvre actuellement `include`, `exclude`, `basePath`, `storybookCompat`et
`inlineArt`. Passez `previewCss`, `previewSetup`, `tokensPath`, `theme`, et `storybookOutDir`
directement à `musea()`.

## Fichiers artistiques

```art-vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./MyButton.vue", {
  title: "MyButton",
  category: "Components",
  status: "ready",
  tags: ["button", "ui", "input"],
});

const pressed = ref(false);
</script>

<art>
  <variant name="Default" default>
    <MyButton type="button" :pressed="pressed">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined :pressed="pressed">Click me</MyButton>
  </variant>
</art>
```

`defineArt(source, options)` est une macro de compilation. Il déclare le composant que Musea doit charger,
plus les métadonnées qui vivaient autrefois sur `<art>`. On préfère une chaîne de chemin à composante relative telle que
`defineArt("./MyButton.vue", { title: "MyButton" })`; Musea importe ce composant dans du code généré
à l’exécution et le serveur de langage utilise la même source pour l’inférence prop et slot.
La chaîne source participe à la complétion de chemin, au diagnostic des fichiers non résolus, aux liens de documents et à
go-to-definition.

`<art title="..." component="...">` fonctionne toujours pour la compatibilité, et les attributs explicites de `<art>`
`defineArt` outrepasser les métadonnées lorsque les deux sont présents.

### État variant local

L’état racine `<script setup>` est isolé par variante par défaut. Chaque variante reçoit sa propre configuration
instance, de sorte que les références et les valeurs calculées d’une variante ne fuient pas dans une autre :

```art-vue
<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
```

Utilisez `<script setup isolate="false">` uniquement lorsque le fichier d’art nécessite intentionnellement une configuration
instance partagée pour chaque variante :

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

### Anatomie

| Élément / Macro                  | Objectif                                                 |
| -------------------------------- | -------------------------------------------------------- |
| `defineArt(source, options)`     | Composant cible et métadonnées artistiques               |
| `defineArt(...).title`           | Nom d’affichage                                          |
| `defineArt(...).category`        | Regroupement dans la barre latérale                      |
| `defineArt(...).status`          | Insigne de statut optionnel                              |
| `defineArt(...).tags`            | Balises de recherche et de filtrage                      |
| `<script setup>`                 | État de configuration variant local par défaut           |
| `<script setup isolate="false">` | État partagé de configuration entre toutes les variantes |
| `<art>`                          | Bloc des variantes racines                               |
| `<art title component ...>`      | Attributs de métadonnées de compatibilité                |
| `<variant>`                      | Variation des composantes nommées                        |
| `default`                        | Marque la variante par défaut                            |
| `args`, `viewport`, `skip-vrt`   | Configuration variante optionnelle                       |

Gardez les fichiers d’art près du composant lorsque les variantes font partie du contrat du composant :

```txt
src/components/Button.vue
src/components/Button.art.vue
```

Utilisez un répertoire `stories` ou `art` séparé lorsqu’un système de conception possède de nombreux exemples transversals,
ou lorsque l’auto-découverte de composants Nuxt scanne le répertoire des composants :

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## Art en ligne

Lorsque `inlineArt` est activé, les fichiers `.vue` classiques contenant un bloc `<art>` peuvent apparaître dans la galerie
. C’est utile pour de petits composants où les exemples doivent être intégrés au même fichier.

```ts
musea({
  inlineArt: true,
});
```

Dans l’art en ligne, utilisez `<Self>` pour rendre le composant hôte.

## Caractéristiques de la galerie

![Musea Component Detail — Variants](/musea-component.png)

La musea peut remonter à la surface :

- Métadonnées composantes et variantes
- Génération de palette d’accessoires
- Vues du jeton de conception
- Contrôles d’accessibilité
- Aides au test de régression visuelle
- Sortie compatible Storybook quand demandée

## Palette d’accessoires

![Musea Props Panel](/musea-props.png)

Le pipeline de palettes peut déduire des contrôles interactifs à partir des métadonnées des composants et des définitions d’art.

## Jetons de conception

![Musea Design Tokens](/musea-tokens.png)

`@vizejs/vite-plugin-musea` peut ingérer un fichier de jeton compatible avec le Dictionnaire de style et l’exposer dans
l’interface de la galerie.

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

## Configuration de l’aperçu

Vous pouvez injecter le CSS du projet et le code de configuration en aperçu :

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

C’est utile pour installer des plugins comme `vue-i18n` ou `vue-router` dans l’iframe de prévisualisation.

```ts
// musea.preview.ts
import type { App } from "vue";
import { createI18n } from "vue-i18n";

export default function setup(app: App) {
  app.use(
    createI18n({
      legacy: false,
      locale: "en",
      messages: {
        en: {},
      },
    }),
  );
}
```

## Test de régression visuelle

Le package expose le `musea-vrt` binaire :

```bash
vp exec musea-vrt --base-url http://localhost:5173
vp exec musea-vrt --update
vp exec musea-vrt --ci --json
vp exec musea-vrt --a11y
vp exec musea-vrt approve
vp exec musea-vrt approve "Button/*"
vp exec musea-vrt clean
```

Le flux CI typique démarre le serveur Vite dans un processus, puis exécute la commande snapshot sur celui-ci :

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

Le flux de travail : valider les lignes de base sous le répertoire snapshot, exécuter `musea-vrt --ci --json` sur un serveur de développement
en marche, puis inspecter `vrt-report.json`/`vrt-report.html` plus `snapshots/current` et
`snapshots/diff` en cas de défaillance. Relancez avec `--update` (ou `approve` pour des variantes sélectionnées) pour
modifications intentionnelles, puis exécutez `clean` après avoir retiré les fichiers d’art afin que les lignes de base obsolètes ne masquent pas les lacunes.
`--ci` sortie non nulle pour les différences visuelles et les erreurs d’aperçu/capture (route manquante, défaillance de
navigateur, délai d’expiration du sélecteur) ; De nouvelles références sont rapportées comme `new`, donc `--update` commence par les effectuer localement.

L’application exemple câble également le chemin VRT natif Playwright (`examples/vite-musea`, exécuté via
`vp run test:vrt` / `vp run test:vrt:update`). Les instantanés vivent dans `e2e/vrt/__snapshots__`, les défauts
les artefacts dans `e2e/vrt/test-results`, et le rapport HTML dans `playwright-report`; GitHub Actions les télécharge
en cas de défaillance afin que les évaluateurs puissent inspecter les images de base, actuelles et différentes.

## Générer des fichiers d’art

Utilisez le générateur pour créer un premier `.art.vue` brouillon à partir d’un composant existant :

```bash
vp exec musea-vrt generate src/components/Button.vue
```

Le fichier généré est un point de départ. Examinez les variantes, titres, tags et la couverture des accessoires avant
de l’engager.

## Production de contes d’histoires

Activez la génération de CSF compatible Storybook lorsque vous souhaitez que des fichiers d’art Musea alimentent une configuration Storybook :

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## Statut CLI

`vize musea` existe dans la ligne de ligne de Rust, mais le flux de travail recommandé de Musea aujourd’hui reste le chemin Vite
plugin. Considérez la sous-commande Rust comme expérimentale pendant que le flux de travail dédié de la galerie se stabilise.

La sous-commande Rust peut enchaîner un projet artistique de départ :

```bash
vize musea new
```

## Paquets associés

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
