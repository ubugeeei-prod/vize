---
title: Accessibility Rules: Structure
---

# Accessibility Rules: Structure

Structure checks keep headings, embedded content, focusable controls, labels, landmarks, and media captions explicit. These pages document every accessibility rule at the same level; there is no
lower-priority "additional" tier, and every rule includes options plus Bad/Good examples.

## `a11y/heading-has-content`

Requires heading elements to have accessible content.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <h2></h2>
</template>
```

Good:

```vue
<template>
  <h2>Billing settings</h2>
</template>
```

## `a11y/heading-levels`

Disallows skipped heading levels so the page outline stays predictable for keyboard and assistive
technology navigation.

Default severity: `warning`
Presets: `nuxt`, `opinionated`
Options: none

Bad:

```vue
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

Good:

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

## `a11y/iframe-has-title`

Requires iframe elements to describe embedded content with `title`.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <iframe src="/checkout"></iframe>
</template>
```

Good:

```vue
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

## `a11y/img-alt`

Requires images to include `alt`. Decorative images should use `alt=""` rather than omitting the
attribute.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <img src="/avatar.png" />
</template>
```

Good:

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/interactive-supports-focus`

Requires elements with interactive roles to be focusable. Native interactive elements are usually
the clearest fix when the semantics match.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

Requires labels to be associated with controls through `for`/`id` or by wrapping the control.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

Good:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/landmark-roles`

Validates landmark placement and uniqueness so page regions are discoverable and not ambiguous.

Default severity: `warning`
Presets: `nuxt`, `opinionated`
Options: none

Bad:

```vue
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

Good:

```vue
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

## `a11y/media-has-caption`

Requires captions or text tracks for audio and video with spoken content.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <video src="/demo.mp4" controls />
</template>
```

Good:

```vue
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```
