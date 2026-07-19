---
title: Règles HTML
---

<!-- Generated translation; source: rules/html.md -->

# Règles HTML

Ces règles couvrent la validité HTML et le balisage sémantique dans les modèles Vue. Elles sont distinctes des règles directives spécifiques à
Vue et des règles d’accessibilité, donc les vérifications de conformité HTML peuvent être activées
ou expliquées seules.

## `html/id-duplication`

Les rapports dulconnent les identifiants statiques à l’intérieur d’un même modèle.

Sévérité par défaut : `error`
Presets : `essential`, `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

Bon :

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

Les rapports ont obsolété les éléments HTML.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <center>Profile</center>
</template>
```

Bon :

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

Les rapports ont déprécié les attributs HTML.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

Bon :

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

Rapporte les éléments consécutifs de `<br>` utilisés pour la mise en page.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

Bon :

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

Nécessite des valeurs de `datetime` lisibles par machine sur `<time>`.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

Bon :

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## `html/no-duplicate-dt`

Les rapports dupliquent `<dt>` termes dans la même `<dl>`.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

Bon :

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

## `html/no-empty-palpable-content`

Signale des éléments vides qui sont censés exposer du contenu visible ou autrement perceptible.
Les éléments contenant du texte, du contenu enfant, `aria-label`, `aria-labelledby`, `v-html`ou `v-text` sont
acceptés.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

Bon :

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```
