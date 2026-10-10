---
title: "Règles Musea et CSS"
---

# Règles Musea et CSS

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="règles-de-la-musea-et-du-css"></span>
<span id="règles-css-supplémentaires"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`css/no-display-none`](#css-no-display-none) | [Mauvais](#css-no-display-none-bad) · [Bon](#css-no-display-none-good) | Suggérer v-show plutôt que display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [Mauvais](#css-no-hardcoded-values-bad) · [Bon](#css-no-hardcoded-values-good) | Suggérer des variables CSS plutôt que des valeurs codées en dur |
| [`css/no-id-selectors`](#css-no-id-selectors) | [Mauvais](#css-no-id-selectors-bad) · [Bon](#css-no-id-selectors-good) | Déconseiller les sélecteurs d’identifiant en CSS |
| [`css/no-important`](#css-no-important) | [Mauvais](#css-no-important-bad) · [Bon](#css-no-important-good) | Déconseiller !important en CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [Mauvais](#css-no-utility-classes-bad) · [Bon](#css-no-utility-classes-good) | Déconseiller l’implémentation de classes utilitaires dans les styles des composants |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [Mauvais](#css-no-v-bind-performance-bad) · [Bon](#css-no-v-bind-performance-good) | Signaler le coût de performance de v-bind() en CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [Mauvais](#css-prefer-logical-properties-bad) · [Bon](#css-prefer-logical-properties-good) | Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [Mauvais](#css-prefer-nested-selectors-bad) · [Bon](#css-prefer-nested-selectors-good) | Recommander l’imbrication CSS pour les sélecteurs de descendants |
| [`css/prefer-slotted`](#css-prefer-slotted) | [Mauvais](#css-prefer-slotted-bad) · [Bon](#css-prefer-slotted-good) | Recommander ::v-slotted() pour styliser le contenu des slots |
| [`css/require-font-display`](#css-require-font-display) | [Mauvais](#css-require-font-display-bad) · [Bon](#css-require-font-display-good) | Exiger font-display dans les règles @font-face |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Mauvais](#musea-no-empty-variant-bad) · [Bon](#musea-no-empty-variant-good) | Interdire les blocs &lt;variant&gt; vides |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Mauvais](#musea-prefer-design-tokens-bad) · [Bon](#musea-prefer-design-tokens-good) | Préférer les variables CSS de design tokens aux valeurs primitives codées en dur |
| [`musea/require-component`](#musea-require-component) | [Mauvais](#musea-require-component-bad) · [Bon](#musea-require-component-good) | Exiger l’attribut component dans le bloc &lt;art&gt; |
| [`musea/require-title`](#musea-require-title) | [Mauvais](#musea-require-title-bad) · [Bon](#musea-require-title-good) | Exiger l’attribut title dans le bloc &lt;art&gt; |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Mauvais](#musea-unique-variant-names-bad) · [Bon](#musea-unique-variant-names-good) | Exiger des noms de variants uniques |
| [`musea/valid-variant`](#musea-valid-variant) | [Mauvais](#musea-valid-variant-bad) · [Bon](#musea-valid-variant-good) | Exiger un attribut name dans les blocs &lt;variant&gt; |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `css/no-display-none`

Suggérer v-show plutôt que display: none

[Mauvais](#css-no-display-none-bad) · [Bon](#css-no-display-none-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**Mauvais**

La déclaration `.message` masque le paragraphe local au moyen du CSS plutôt que d’une condition de visibilité dans le template.

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**Bon**

`v-show="isSaved"` rend la condition de visibilité explicite sur le paragraphe local et supprime `display: none`.

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [Toutes les règles](all.md)

### `css/no-hardcoded-values`

Suggérer des variables CSS plutôt que des valeurs codées en dur

[Mauvais](#css-no-hardcoded-values-bad) · [Bon](#css-no-hardcoded-values-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**Mauvais**

Le bouton place directement des nombres d’espacement et une couleur hexadécimale dans les déclarations.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**Bon**

Les déclarations référencent des propriétés personnalisées d’espacement et de couleur nommées, afin que ces valeurs puissent être maintenues sous forme de tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [Toutes les règles](all.md)

### `css/no-id-selectors`

Déconseiller les sélecteurs d’identifiant en CSS

[Mauvais](#css-no-id-selectors-bad) · [Bon](#css-no-id-selectors-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**Mauvais**

`#submit` lie la règle de style à un sélecteur d’identifiant.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**Bon**

La classe `.submit` fournit un point d’accroche de style réutilisable sans sélecteur d’identifiant.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [Toutes les règles](all.md)

### `css/no-important`

Déconseiller !important en CSS

[Mauvais](#css-no-important-bad) · [Bon](#css-no-important-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**Mauvais**

La déclaration de couleur remplace la priorité normale de la cascade avec `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**Bon**

La couleur provient d’une propriété personnalisée sans déclaration important.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [Toutes les règles](all.md)

### `css/no-utility-classes`

Déconseiller l’implémentation de classes utilitaires dans les styles des composants

[Mauvais](#css-no-utility-classes-bad) · [Bon](#css-no-utility-classes-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**Mauvais**

Les sélecteurs écrits utilisent des noms de forme utilitaire, tels que `.flex`, `.mt-4` et `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**Bon**

Un sélecteur `.my-component` propre au composant rassemble ses styles sous un nom sémantique unique.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [Toutes les règles](all.md)

### `css/no-v-bind-performance`

Signaler le coût de performance de v-bind() en CSS

[Mauvais](#css-no-v-bind-performance-bad) · [Bon](#css-no-v-bind-performance-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**Mauvais**

La feuille de style lit la valeur changeante `offset` au moyen du mécanisme CSS `v-bind()` du SFC.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**Bon**

L’élément reçoit directement la transformation changeante via sa liaison de style.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [Toutes les règles](all.md)

### `css/prefer-logical-properties`

Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation

[Mauvais](#css-prefer-logical-properties-bad) · [Bon](#css-prefer-logical-properties-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**Mauvais**

`margin-left` fixe la marge sur un côté physique indépendamment du sens d’écriture.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**Bon**

`margin-inline-start` suit plutôt le début de la direction en ligne.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [Toutes les règles](all.md)

### `css/prefer-nested-selectors`

Recommander l’imbrication CSS pour les sélecteurs de descendants

[Mauvais](#css-prefer-nested-selectors-bad) · [Bon](#css-prefer-nested-selectors-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**Mauvais**

Le sélecteur de descendant `.card .title` répète le sélecteur parent dans une règle à plat.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**Bon**

La règle `.title` est imbriquée dans `.card`, ce qui rassemble la relation de style parent-enfant.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [Toutes les règles](all.md)

### `css/prefer-slotted`

Recommander ::v-slotted() pour styliser le contenu des slots

[Mauvais](#css-prefer-slotted-bad) · [Bon](#css-prefer-slotted-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**Mauvais**

La feuille de style scoped cible le point d’insertion `slot` plutôt que les éléments fournis au slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**Bon**

`:slotted(.label)` cible l’élément label fourni au moyen du sélecteur de slot scoped.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [Toutes les règles](all.md)

### `css/require-font-display`

Exiger font-display dans les règles @font-face

[Mauvais](#css-require-font-display-bad) · [Bon](#css-require-font-display-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**Mauvais**

La déclaration font-face définit la source de la police, mais omet sa politique font-display.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**Bon**

`font-display: swap` sélectionne explicitement la politique d’affichage passant d’une police de repli à la police chargée.

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [Toutes les règles](all.md)

### `musea/no-empty-variant`

Interdire les blocs &lt;variant&gt; vides

[Mauvais](#musea-no-empty-variant-bad) · [Bon](#musea-no-empty-variant-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**Mauvais**

Le variant nommé primary est vide et ne fournit donc aucun contenu d’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**Bon**

Le variant affiche un Button primary avec son contenu Save.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [Toutes les règles](all.md)

### `musea/prefer-design-tokens`

Préférer les variables CSS de design tokens aux valeurs primitives codées en dur

[Mauvais](#musea-prefer-design-tokens-bad) · [Bon](#musea-prefer-design-tokens-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Nécessite un fichier .art.vue et l’inventaire des tokens présenté ci-dessous. Aucun token n’est déduit d’une couleur arbitraire.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-prefer-design-tokens-bad"></span>

**Mauvais**

L’exemple art utilise la couleur bleue littérale au lieu du design token primaire configuré.

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**Bon**

Le style référence --color-primary, le token configuré pour cet exemple.

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [Toutes les règles](all.md)

### `musea/require-component`

Exiger l’attribut component dans le bloc &lt;art&gt;

[Mauvais](#musea-require-component-bad) · [Bon](#musea-require-component-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**Mauvais**

Le bloc art fournit un titre, mais n’identifie pas le composant dont il présente l’aperçu.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**Bon**

defineArt fournit ./Button.vue comme composant du bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [Toutes les règles](all.md)

### `musea/require-title`

Exiger l’attribut title dans le bloc &lt;art&gt;

[Mauvais](#musea-require-title-bad) · [Bon](#musea-require-title-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**Mauvais**

Le bloc art identifie Button.vue, mais ne fournit aucun titre.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**Bon**

Les options de defineArt fournissent le titre Button pour le bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [Toutes les règles](all.md)

### `musea/unique-variant-names`

Exiger des noms de variants uniques

[Mauvais](#musea-unique-variant-names-bad) · [Bon](#musea-unique-variant-names-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**Mauvais**

Deux variants du même bloc art utilisent tous deux le nom primary.

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**Bon**

Les variants possèdent les noms distincts primary et secondary.

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [Toutes les règles](all.md)

### `musea/valid-variant`

Exiger un attribut name dans les blocs &lt;variant&gt;

[Mauvais](#musea-valid-variant-bad) · [Bon](#musea-valid-variant-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**Mauvais**

Le variant omet le nom nécessaire pour identifier l’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**Bon**

Le nom primary identifie ce variant.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [Toutes les règles](all.md)
