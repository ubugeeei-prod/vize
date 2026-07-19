---
title: Règles d’accessibilité
---

<!-- Generated translation; source: rules/accessibility.md -->

# Règles d’accessibilité

Les règles d’accessibilité sont des règles modèles en file indienne Patina. Ils détectent des balises difficiles à utiliser
avec la technologie d’assistance ou la navigation au clavier.

## `a11y/img-alt`

Nécessite un attribut `alt` sur `<img>`.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <img src="/avatar.png" />
</template>
```

Bon :

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/alt-text`

Nécessite un texte alternatif pour les éléments médias qui nécessitent une alternative textuelle.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

Bon :

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit" />
</template>
```

## `a11y/click-events-have-key-events`

Signale les gestionnaires de clics sur des éléments interactifs non natifs lorsqu’aucun gestionnaire de clavier n’est présent.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

Bon :

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/interactive-supports-focus`

Il faut que les éléments avec des rôles interactifs soient ciblables.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

Bon :

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

Nécessite que les étiquettes soient associées à un contrôle de formulaire.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

Bon :

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/form-control-has-label`

Nécessite que les contrôles aient une étiquette visible ou programmatique.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <input type="search" />
</template>
```

Bon :

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

Signale des éléments ciblés cachés à la technologie d’assistance.

Sévérité par défaut : `error`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

Bon :

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-static-element-interactions`

Signale les manipulateurs de souris ou de clavier sur des éléments statiques.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <section @click="select">Select</section>
</template>
```

Bon :

```vue
<template>
  <button type="button" @click="select">Select</button>
</template>
```

## `a11y/tabindex-no-positive`

Rapporte des valeurs de `tabindex` positives car elles créent un ordre de tabulation personnalisé difficile à prévoir.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

Bon :

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/anchor-is-valid`

Nécessite que les ancres aient des cibles de liaison valides.
Les valeurs de `href` statiques sont vérifiées après normalisation du schéma, donc les `JaVaScRiPt:` et les caractères de contrôle de
décodés en HTML à l’intérieur de `java&#x0A;script:` sont toujours rapportés tandis que des schémas similaires non correspondants
rester autorisés.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <a href="#" @click="open">Open</a>
  <a href="JaVaScRiPt:void(0)">Open</a>
</template>
```

Bon :

```vue
<template>
  <button type="button" @click="open">Open</button>
  <a href="/docs/javascript:void">Docs</a>
</template>
```

## Règles supplémentaires d’accessibilité

`a11y/anchor-has-content` exige que les éléments d’ancrage aient un contenu accessible. Par défaut : `warning`.
Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/aria-props` interdit les attributs ARIA invalides. Par défaut : `error`. Presets : `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-role` nécessite des rôles ARIA valides et non abstraits. Par défaut : `error`. Presets : `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-unsupported-elements` interdit les attributs ARIA sur les éléments qui ne les supportent pas.
Par défaut : `error`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-has-content` nécessite que les éléments de titre aient un contenu accessible. Par défaut : `warning`.
Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-levels` interdit les niveaux de tête sautés. Par défaut : `warning`. Presets : `nuxt`,
`opinionated`.

`a11y/iframe-has-title` exige `<iframe>` avoir un `title`. Par défaut : `warning`. Préréglages :
`happy-path`, `nuxt`, `opinionated`.

`a11y/landmark-roles` valide la position et l’unicité des rôles emblématiques. Par défaut : `warning`.
Presets : `nuxt`, `opinionated`.

`a11y/media-has-caption` nécessite des légendes pour les éléments médias. Par défaut : `warning`. Préréglages :
`happy-path`, `nuxt`, `opinionated`.

`a11y/mouse-events-have-key-events` nécessite des manipulateurs de mise au point et de flou lorsque les manipulateurs de souris sont utilisés.
Par défaut : `warning`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/no-access-key` interdit l’attribut `accesskey`. Par défaut : `warning`. Presets :
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-autofocus` interdit `autofocus`. Par défaut : `warning`. Presets : `happy-path`, `nuxt`,
`opinionated`.

`a11y/no-distracting-elements` interdit les éléments distrayants tels que `<marquee>` et `<blink>`.
Par défaut : `warning`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/no-i-for-icon` décourage d’utiliser `<i>` comme élément uniquement d’icônes. Par défaut : `warning`. Presets :
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-redundant-roles` interdit les rôles ARIA qui dupliquent la sémantique native. Par défaut :
`warning`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/no-refer-to-non-existent-id` signale des références ARIA à des pièces d’identité manquantes. Par défaut : `warning`.
Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/no-role-presentation-on-focusable` interdit `role="presentation"` ou `role="none"` sur
éléments de mise au point. Par défaut : `error`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/placeholder-label-option` nécessite d’être désactivé ou caché sur les valeurs de `<option>` de placeholder.
Par défaut : `warning`. Presets : `nuxt`, `opinionated`.

`a11y/role-has-required-aria-props` exige que les rôles incluent leurs attributs ARIA requis.
Par défaut : `warning`. Presets : `happy-path`, `nuxt`, `opinionated`.

`a11y/use-list` suggère des éléments de liste pour un texte en forme de puces. Par défaut : `warning`. Presets : `nuxt`,
`opinionated`.
