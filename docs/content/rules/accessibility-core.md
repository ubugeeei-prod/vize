---
title: Accessibility Rules: Core Markup
---

# Accessibility Rules: Core Markup

Core accessibility checks cover missing names, invalid links, ARIA basics, keyboard activation, and form labels. These pages document every accessibility rule at the same level; there is no
lower-priority "additional" tier, and every rule includes options plus Bad/Good examples.

## `a11y/alt-text`

Requires text alternatives for media elements such as image inputs, `area`, `object`, and SVG
`image`. Decorative imagery still needs an explicit empty alternative when the element type supports
one, so reviewers can tell the omission was intentional.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

Good:

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

## `a11y/anchor-has-content`

Requires anchors to expose visible text, an accessible label, or labelled children. Empty links are
announced without useful context and become mystery stops in keyboard navigation.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <a href="/settings"></a>
</template>
```

Good:

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

## `a11y/anchor-is-valid`

Requires anchors to have real link targets. Static `href` values are normalized before checking, so
mixed-case `javascript:` URLs and decoded control characters are still rejected.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

## `a11y/aria-props`

Disallows invalid or misspelled ARIA attributes before they silently disappear from the
accessibility tree.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

Good:

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

## `a11y/aria-role`

Requires ARIA roles to be valid concrete roles. Unknown and abstract roles do not create the
semantics authors expect.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

Good:

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

## `a11y/aria-unsupported-elements`

Disallows ARIA attributes on elements that cannot expose ARIA semantics.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

Good:

```vue
<template>
  <meta charset="utf-8" />
</template>
```

## `a11y/click-events-have-key-events`

Requires keyboard access when click handlers are used on non-native interactive elements. Prefer a
native control when the intended behavior already exists in HTML.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/form-control-has-label`

Requires form controls to have a visible label or programmatic accessible name.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <input type="search" />
</template>
```

Good:

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```
