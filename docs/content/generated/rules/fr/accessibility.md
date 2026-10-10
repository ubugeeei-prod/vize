---
title: "Règles d’accessibilité"
---

# Règles d’accessibilité

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="règles-supplémentaires-d-accessibilité"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [Mauvais](#a11y-alt-text-bad) · [Bon](#a11y-alt-text-good) | Exiger un texte alternatif pour les éléments multimédias |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [Mauvais](#a11y-anchor-has-content-bad) · [Bon](#a11y-anchor-has-content-good) | Exiger un contenu accessible pour les éléments de lien |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [Mauvais](#a11y-anchor-is-valid-bad) · [Bon](#a11y-anchor-is-valid-good) | Imposer un href valide sur les éléments de lien |
| [`a11y/aria-props`](#a11y-aria-props) | [Mauvais](#a11y-aria-props-bad) · [Bon](#a11y-aria-props-good) | Interdire les attributs ARIA invalides |
| [`a11y/aria-role`](#a11y-aria-role) | [Mauvais](#a11y-aria-role-bad) · [Bon](#a11y-aria-role-good) | Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [Mauvais](#a11y-aria-unsupported-elements-bad) · [Bon](#a11y-aria-unsupported-elements-good) | Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [Mauvais](#a11y-click-events-have-key-events-bad) · [Bon](#a11y-click-events-have-key-events-good) | Exiger des gestionnaires de clavier avec les événements de clic |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [Mauvais](#a11y-form-control-has-label-bad) · [Bon](#a11y-form-control-has-label-good) | Exiger des libellés associés aux contrôles de formulaire |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Mauvais](#a11y-heading-has-content-bad) · [Bon](#a11y-heading-has-content-good) | Exiger un contenu accessible pour les éléments de titre |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Mauvais](#a11y-heading-levels-bad) · [Bon](#a11y-heading-levels-good) | Interdire de sauter des niveaux de titre |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Mauvais](#a11y-iframe-has-title-bad) · [Bon](#a11y-iframe-has-title-good) | Exiger un attribut title sur les éléments iframe |
| [`a11y/img-alt`](#a11y-img-alt) | [Mauvais](#a11y-img-alt-bad) · [Bon](#a11y-img-alt-good) | Exiger un attribut alt sur les images pour les rendre accessibles |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Mauvais](#a11y-interactive-supports-focus-bad) · [Bon](#a11y-interactive-supports-focus-good) | Exiger que les éléments possédant un rôle interactif puissent recevoir le focus |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Mauvais](#a11y-label-has-for-bad) · [Bon](#a11y-label-has-for-good) | Exiger des contrôles de formulaire associés aux libellés |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Mauvais](#a11y-landmark-roles-bad) · [Bon](#a11y-landmark-roles-good) | Valider l’emplacement et l’unicité des rôles de zones de repère |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Mauvais](#a11y-media-has-caption-bad) · [Bon](#a11y-media-has-caption-good) | Exiger des sous-titres pour les éléments multimédias |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Mauvais](#a11y-mouse-events-have-key-events-bad) · [Bon](#a11y-mouse-events-have-key-events-good) | Exiger des événements de focus et de perte du focus avec les événements de souris |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Mauvais](#a11y-no-access-key-bad) · [Bon](#a11y-no-access-key-good) | Interdire l’utilisation de l’attribut accesskey |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Mauvais](#a11y-no-aria-hidden-on-focusable-bad) · [Bon](#a11y-no-aria-hidden-on-focusable-good) | Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Mauvais](#a11y-no-autofocus-bad) · [Bon](#a11y-no-autofocus-good) | Interdire l’utilisation de l’attribut autofocus |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Mauvais](#a11y-no-distracting-elements-bad) · [Bon](#a11y-no-distracting-elements-good) | Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt; |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Mauvais](#a11y-no-i-for-icon-bad) · [Bon](#a11y-no-i-for-icon-good) | Interdire l’élément &lt;i&gt; pour les icônes |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Mauvais](#a11y-no-redundant-roles-bad) · [Bon](#a11y-no-redundant-roles-good) | Interdire les rôles ARIA redondants |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Mauvais](#a11y-no-refer-to-non-existent-id-bad) · [Bon](#a11y-no-refer-to-non-existent-id-good) | Interdire les références à des identifiants inexistants |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Mauvais](#a11y-no-role-presentation-on-focusable-bad) · [Bon](#a11y-no-role-presentation-on-focusable-good) | Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Mauvais](#a11y-no-static-element-interactions-bad) · [Bon](#a11y-no-static-element-interactions-good) | Interdire les gestionnaires d’événements sur les éléments statiques |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Mauvais](#a11y-placeholder-label-option-bad) · [Bon](#a11y-placeholder-label-option-good) | Exiger disabled ou hidden sur l’option d’invite d’un select |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Mauvais](#a11y-role-has-required-aria-props-bad) · [Bon](#a11y-role-has-required-aria-props-good) | Exiger les propriétés obligatoires des rôles ARIA |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Mauvais](#a11y-tabindex-no-positive-bad) · [Bon](#a11y-tabindex-no-positive-good) | Interdire les valeurs positives de tabindex |
| [`a11y/use-list`](#a11y-use-list) | [Mauvais](#a11y-use-list-bad) · [Bon](#a11y-use-list-good) | Suggérer des éléments de liste pour les textes ressemblant à des listes à puces |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good) | Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Exiger un texte alternatif pour les éléments multimédias

[Mauvais](#a11y-alt-text-bad) · [Bon](#a11y-alt-text-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**Mauvais**

Le contrôle de soumission sous forme d’image ne fournit que l’URL de son image ; il n’a pas de texte `alt` décrivant l’action.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**Bon**

`alt="Submit search"` donne au contrôle image un nom accessible décrivant la soumission de la recherche.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [Toutes les règles](all.md)

### `a11y/anchor-has-content`

Exiger un contenu accessible pour les éléments de lien

[Mauvais](#a11y-anchor-has-content-bad) · [Bon](#a11y-anchor-has-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**Mauvais**

Le lien `/settings` n’a ni texte ni autre contenu servant à le nommer ; sa destination n’a donc aucune description accessible.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**Bon**

Le texte visible `Settings` fournit le contenu du lien vers la même destination.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [Toutes les règles](all.md)

### `a11y/anchor-is-valid`

Imposer un href valide sur les éléments de lien

[Mauvais](#a11y-anchor-is-valid-bad) · [Bon](#a11y-anchor-is-valid-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**Mauvais**

Le premier lien utilise `#` pour une action ; le second utilise une URL JavaScript. Aucun ne fournit une destination de navigation ordinaire.

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**Bon**

Un bouton natif exécute `openPanel`, tandis que le lien restant possède la destination réelle `/docs/javascript-urls`.

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [Toutes les règles](all.md)

### `a11y/aria-props`

Interdire les attributs ARIA invalides

[Mauvais](#a11y-aria-props-bad) · [Bon](#a11y-aria-props-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**Mauvais**

`aria-lable` est mal orthographié et n’est pas un attribut ARIA pris en charge.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**Bon**

L’attribut pris en charge `aria-label` fournit le nom du bouton.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [Toutes les règles](all.md)

### `a11y/aria-role`

Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA

[Mauvais](#a11y-aria-role-bad) · [Bon](#a11y-aria-role-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**Mauvais**

`datepicker` n’est pas un rôle ARIA reconnu pour cette section.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**Bon**

La section utilise le rôle reconnu `dialog` et un libellé décrivant la sélection de la date.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [Toutes les règles](all.md)

### `a11y/aria-unsupported-elements`

Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge

[Mauvais](#a11y-aria-unsupported-elements-bad) · [Bon](#a11y-aria-unsupported-elements-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**Mauvais**

L’élément de métadonnées porte `aria-hidden`, alors que `meta` ne prend pas en charge les attributs ARIA.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**Bon**

Supprimer l’attribut ARIA conserve intacte la déclaration de jeu de caractères.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [Toutes les règles](all.md)

### `a11y/click-events-have-key-events`

Exiger des gestionnaires de clavier avec les événements de clic

[Mauvais](#a11y-click-events-have-key-events-bad) · [Bon](#a11y-click-events-have-key-events-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Contrôle les éléments non interactifs sans rôle interactif. Les boutons natifs et les éléments possédant un rôle ARIA interactif sont hors du champ du diagnostic de cette règle.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**Mauvais**

Le `div` non interactif possède un gestionnaire de clic, mais aucune gestion des événements de clavier.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**Bon**

Un `button` natif permet une activation au clavier avec le même gestionnaire `activate`.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [Toutes les règles](all.md)

### `a11y/form-control-has-label`

Exiger des libellés associés aux contrôles de formulaire

[Mauvais](#a11y-form-control-has-label-bad) · [Bon](#a11y-form-control-has-label-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**Mauvais**

Le champ de recherche n’a pas de libellé indiquant ce que l’utilisateur doit saisir.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**Bon**

Envelopper le champ dans un label associe le texte visible `Search` au contrôle.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [Toutes les règles](all.md)

### `a11y/heading-has-content`

Exiger un contenu accessible pour les éléments de titre

[Mauvais](#a11y-heading-has-content-bad) · [Bon](#a11y-heading-has-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**Mauvais**

Le `h2` apporte un niveau de titre, mais ne contient aucun texte de titre.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**Bon**

`Billing settings` fournit le contenu du titre de niveau deux existant.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [Toutes les règles](all.md)

### `a11y/heading-levels`

Interdire de sauter des niveaux de titre

[Mauvais](#a11y-heading-levels-bad) · [Bon](#a11y-heading-levels-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**Mauvais**

La séquence des titres passe directement de `h1` à `h3`, sans niveau deux.

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**Bon**

Remplacer le titre de facturation par `h2` conserve une hiérarchie de titres consécutive.

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [Toutes les règles](all.md)

### `a11y/iframe-has-title`

Exiger un attribut title sur les éléments iframe

[Mauvais](#a11y-iframe-has-title-bad) · [Bon](#a11y-iframe-has-title-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**Mauvais**

Le cadre de paiement possède une URL source, mais aucun `title` décrivant le contenu intégré.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**Bon**

`title="Checkout preview"` nomme le contenu de ce cadre.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [Toutes les règles](all.md)

### `a11y/img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

[Mauvais](#a11y-img-alt-bad) · [Bon](#a11y-img-alt-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**Mauvais**

L’image de l’avatar ne possède pas d’attribut `alt`.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**Bon**

`alt="User avatar"` fournit une alternative textuelle à l’avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [Toutes les règles](all.md)

### `a11y/interactive-supports-focus`

Exiger que les éléments possédant un rôle interactif puissent recevoir le focus

[Mauvais](#a11y-interactive-supports-focus-bad) · [Bon](#a11y-interactive-supports-focus-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**Mauvais**

Donner à un `span` le rôle button et un gestionnaire de clic ne rend pas l’élément accessible au focus clavier.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**Bon**

Le bouton natif peut recevoir le focus et conserve la même action `open`.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [Toutes les règles](all.md)

### `a11y/label-has-for`

Exiger des contrôles de formulaire associés aux libellés

[Mauvais](#a11y-label-has-for-bad) · [Bon](#a11y-label-has-for-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**Mauvais**

Le label séparé n’est ni associé au moyen de `for` ni placé autour du champ.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**Bon**

`for="email"` correspond à l’identifiant du champ et associe explicitement les deux éléments.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [Toutes les règles](all.md)

### `a11y/landmark-roles`

Valider l’emplacement et l’unicité des rôles de zones de repère

[Mauvais](#a11y-landmark-roles-bad) · [Bon](#a11y-landmark-roles-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**Mauvais**

Deux éléments `main` déclarent des zones principales en double dans le même template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**Bon**

Le tableau de bord reste la zone principale ; la zone des paramètres devient une zone de navigation nommée.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [Toutes les règles](all.md)

### `a11y/media-has-caption`

Exiger des sous-titres pour les éléments multimédias

[Mauvais](#a11y-media-has-caption-bad) · [Bon](#a11y-media-has-caption-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**Mauvais**

La vidéo possède des contrôles de lecture, mais aucune piste de sous-titres.

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**Bon**

Un `track` avec `kind="captions"` fournit les sous-titres anglais de la même vidéo.

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [Toutes les règles](all.md)

### `a11y/mouse-events-have-key-events`

Exiger des événements de focus et de perte du focus avec les événements de souris

[Mauvais](#a11y-mouse-events-have-key-events-bad) · [Bon](#a11y-mouse-events-have-key-events-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**Mauvais**

La visibilité de l’aperçu change uniquement avec les gestionnaires d’entrée et de sortie de la souris.

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**Bon**

Les mêmes actions d’aperçu s’exécutent lors du focus et de sa perte, et le bouton peut recevoir le focus clavier.

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [Toutes les règles](all.md)

### `a11y/no-access-key`

Interdire l’utilisation de l’attribut accesskey

[Mauvais](#a11y-no-access-key-bad) · [Bon](#a11y-no-access-key-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**Mauvais**

Le raccourci `accesskey="s"` peut entrer en conflit avec les raccourcis du navigateur ou des technologies d’assistance.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**Bon**

Supprimer `accesskey` conserve le bouton Save ordinaire.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-aria-hidden-on-focusable`

Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus

[Mauvais](#a11y-no-aria-hidden-on-focusable-bad) · [Bon](#a11y-no-aria-hidden-on-focusable-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**Mauvais**

Le bouton Close pouvant recevoir le focus est masqué dans l’arbre d’accessibilité avec `aria-hidden="true"`.

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**Bon**

Le bouton reste exposé et reçoit un libellé `Close` au lieu d’être masqué.

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-autofocus`

Interdire l’utilisation de l’attribut autofocus

[Mauvais](#a11y-no-autofocus-bad) · [Bon](#a11y-no-autofocus-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**Mauvais**

Le champ demande à recevoir automatiquement le focus lorsqu’il apparaît.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**Bon**

Supprimer `autofocus` évite cette demande de focus automatique tout en conservant le champ de requête.

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-distracting-elements`

Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;

[Mauvais](#a11y-no-distracting-elements-bad) · [Bon](#a11y-no-distracting-elements-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**Mauvais**

L’élément `marquee` introduit un texte en mouvement automatique.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**Bon**

Un paragraphe affiche la même offre sans l’élément marquee distrayant.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [Toutes les règles](all.md)

### `a11y/no-i-for-icon`

Interdire l’élément &lt;i&gt; pour les icônes

[Mauvais](#a11y-no-i-for-icon-bad) · [Bon](#a11y-no-i-for-icon-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**Mauvais**

L’icône est affichée avec `i`, dont la sémantique textuelle ne décrit pas une action représentée uniquement par une icône.

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**Bon**

Un span décoratif masque le glyphe de l’icône, tandis que le texte séparé `Delete item` nomme l’action du bouton.

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [Toutes les règles](all.md)

### `a11y/no-redundant-roles`

Interdire les rôles ARIA redondants

[Mauvais](#a11y-no-redundant-roles-bad) · [Bon](#a11y-no-redundant-roles-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**Mauvais**

Le bouton natif possède déjà le rôle button ; `role="button"` répète donc sa sémantique implicite.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**Bon**

Supprimer le rôle répété conserve la sémantique du bouton fournie par HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [Toutes les règles](all.md)

### `a11y/no-refer-to-non-existent-id`

Interdire les références à des identifiants inexistants

[Mauvais](#a11y-no-refer-to-non-existent-id-bad) · [Bon](#a11y-no-refer-to-non-existent-id-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**Mauvais**

`aria-labelledby` pointe vers `save-label`, mais aucun élément ne déclare cet identifiant.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**Bon**

Ajouter le span correspondant résout la référence et fournit le libellé du bouton.

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [Toutes les règles](all.md)

### `a11y/no-role-presentation-on-focusable`

Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus

[Mauvais](#a11y-no-role-presentation-on-focusable-bad) · [Bon](#a11y-no-role-presentation-on-focusable-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**Mauvais**

Le lien de facturation pouvant recevoir le focus demande role=presentation, ce qui entre en conflit avec son rôle de lien interactif ; les navigateurs doivent ignorer cette demande de présentation.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**Bon**

Supprimez la demande de présentation contradictoire et utilisez le rôle de lien natif ainsi que la destination de facturation.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-static-element-interactions`

Interdire les gestionnaires d’événements sur les éléments statiques

[Mauvais](#a11y-no-static-element-interactions-bad) · [Bon](#a11y-no-static-element-interactions-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**Mauvais**

Une section statique reçoit une action liée à la touche Entrée sans posséder de rôle interactif.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**Bon**

Un bouton natif porte la même action dans un élément interactif approprié.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [Toutes les règles](all.md)

### `a11y/placeholder-label-option`

Exiger disabled ou hidden sur l’option d’invite d’un select

[Mauvais](#a11y-placeholder-label-option-bad) · [Bon](#a11y-placeholder-label-option-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**Mauvais**

L’invite à valeur vide reste sélectionnable comme s’il s’agissait d’une valeur de pays.

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**Bon**

Ajouter `disabled` distingue l’invite de l’option Japan sélectionnable.

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [Toutes les règles](all.md)

### `a11y/role-has-required-aria-props`

Exiger les propriétés obligatoires des rôles ARIA

[Mauvais](#a11y-role-has-required-aria-props-bad) · [Bon](#a11y-role-has-required-aria-props-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**Mauvais**

Le rôle checkbox omet `aria-checked`, qui communique l’état de la case à cocher.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**Bon**

`aria-checked="false"` fournit l’état exigé par le rôle checkbox.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [Toutes les règles](all.md)

### `a11y/tabindex-no-positive`

Interdire les valeurs positives de tabindex

[Mauvais](#a11y-tabindex-no-positive-bad) · [Bon](#a11y-tabindex-no-positive-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**Mauvais**

Un tabindex positif de 3 crée un ordre de focus personnalisé avant les contrôles ordinaires.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**Bon**

Le bouton utilise son ordre de focus natif sans tabindex positif.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [Toutes les règles](all.md)

### `a11y/use-list`

Suggérer des éléments de liste pour les textes ressemblant à des listes à puces

[Mauvais](#a11y-use-list-bad) · [Bon](#a11y-use-list-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**Mauvais**

Les tâches sont des paragraphes séparés précédés de tirets saisis, plutôt que des éléments de liste.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**Bon**

Une liste non ordonnée et ses éléments expriment les mêmes tâches avec une sémantique de liste.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [Toutes les règles](all.md)

### `vue/use-unique-element-ids`

Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques

[Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**Mauvais**

L’identifiant littéral `email` est réutilisé par chaque instance de ce composant, ce qui peut faire pointer son label vers la mauvaise instance lorsque plusieurs sont affichées.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Bon**

`useId()` produit l’`emailId` de l’instance ; liez la même valeur au `for` du label et à l’`id` de l’input.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [Toutes les règles](all.md)
