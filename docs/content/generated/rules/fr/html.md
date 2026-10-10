---
title: "Règles HTML"
---

# Règles HTML

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Mauvais](#html-deprecated-attr-bad) · [Bon](#html-deprecated-attr-good) | Interdire les attributs HTML obsolètes |
| [`html/deprecated-element`](#html-deprecated-element) | [Mauvais](#html-deprecated-element-bad) · [Bon](#html-deprecated-element-good) | Interdire les éléments HTML obsolètes |
| [`html/id-duplication`](#html-id-duplication) | [Mauvais](#html-id-duplication-bad) · [Bon](#html-id-duplication-good) | Interdire les identifiants d’éléments en double |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Mauvais](#html-no-consecutive-br-bad) · [Bon](#html-no-consecutive-br-good) | Interdire les éléments &lt;br&gt; consécutifs |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Mauvais](#html-no-dupe-style-properties-bad) · [Bon](#html-no-dupe-style-properties-good) | Interdire les propriétés en double dans les attributs de style en ligne |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Mauvais](#html-no-duplicate-class-bad) · [Bon](#html-no-duplicate-class-good) | Interdire les noms de classes en double dans un attribut class statique |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Mauvais](#html-no-duplicate-dt-bad) · [Bon](#html-no-duplicate-dt-good) | Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt; |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Mauvais](#html-no-empty-palpable-content-bad) · [Bon](#html-no-empty-palpable-content-good) | Interdire les éléments vides qui attendent un contenu visible |
| [`html/require-datetime`](#html-require-datetime) | [Mauvais](#html-require-datetime-bad) · [Bon](#html-require-datetime-good) | Exiger l’attribut datetime sur l’élément &lt;time&gt; |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `html/deprecated-attr`

Interdire les attributs HTML obsolètes

[Mauvais](#html-deprecated-attr-bad) · [Bon](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**Mauvais**

Le paragraphe utilise l’attribut de présentation obsolète `align`.

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**Bon**

La classe et la déclaration `text-align: center` expriment l’alignement au moyen du CSS.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [Toutes les règles](all.md)

### `html/deprecated-element`

Interdire les éléments HTML obsolètes

[Mauvais](#html-deprecated-element-bad) · [Bon](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**Mauvais**

L’élément `center` utilise un élément de présentation HTML obsolète.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**Bon**

Une section et une classe de style remplacent l’élément obsolète tout en préservant le contenu.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [Toutes les règles](all.md)

### `html/id-duplication`

Interdire les identifiants d’éléments en double

[Mauvais](#html-id-duplication-bad) · [Bon](#html-id-duplication-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**Mauvais**

Le champ et le paragraphe d’aide déclarent tous deux `id="email"`, rendant la cible du label ambiguë.

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**Bon**

Le champ conserve `email` ; le paragraphe d’aide utilise `email-help`, et aria-describedby référence cet identifiant distinct.

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [Toutes les règles](all.md)

### `html/no-consecutive-br`

Interdire les éléments &lt;br&gt; consécutifs

[Mauvais](#html-no-consecutive-br-bad) · [Bon](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**Mauvais**

Deux éléments de saut de ligne consécutifs créent un espacement entre des blocs au sein d’un seul paragraphe.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**Bon**

Des paragraphes séparés expriment les deux blocs de contenu sans répéter les éléments de saut de ligne.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [Toutes les règles](all.md)

### `html/no-dupe-style-properties`

Interdire les propriétés en double dans les attributs de style en ligne

[Mauvais](#html-no-dupe-style-properties-bad) · [Bon](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**Mauvais**

Chaque style statique répète une propriété ; `margin` et `MARGIN` sont également considérés comme la même propriété.

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**Bon**

Le style statique utilise des propriétés distinctes de couleur et d’arrière-plan. Les liaisons de style dynamiques sont hors du champ de ce contrôle des attributs statiques.

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [Toutes les règles](all.md)

### `html/no-duplicate-class`

Interdire les noms de classes en double dans un attribut class statique

[Mauvais](#html-no-duplicate-class-bad) · [Bon](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**Mauvais**

La liste de classes statique répète le nom `btn`.

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**Bon**

La liste de classes conserve une seule occurrence de `btn` et le nom distinct `primary`.

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [Toutes les règles](all.md)

### `html/no-duplicate-dt`

Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;

[Mauvais](#html-no-duplicate-dt-bad) · [Bon](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**Mauvais**

La même liste de définitions répète le terme `API` pour deux descriptions.

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**Bon**

Un seul terme API est suivi des deux descriptions, évitant de répéter le terme.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [Toutes les règles](all.md)

### `html/no-empty-palpable-content`

Interdire les éléments vides qui attendent un contenu visible

[Mauvais](#html-no-empty-palpable-content-bad) · [Bon](#html-no-empty-palpable-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**Mauvais**

Le paragraphe, l’élément de liste et la cellule de tableau ont tous un contenu palpable vide.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**Bon**

Du texte remplit le paragraphe, une interpolation fournit le contenu de l’élément de liste, et aria-label nomme explicitement la cellule autrement vide.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [Toutes les règles](all.md)

### `html/require-datetime`

Exiger l’attribut datetime sur l’élément &lt;time&gt;

[Mauvais](#html-require-datetime-bad) · [Bon](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**Mauvais**

L’élément time contient une date lisible par une personne, mais aucune valeur datetime lisible par une machine.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**Bon**

`datetime="2026-05-13"` fournit la date correspondante lisible par une machine.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [Toutes les règles](all.md)
