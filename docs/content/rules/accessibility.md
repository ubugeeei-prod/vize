---
title: Accessibility Rules
---

# Accessibility Rules

Accessibility rules are Patina single-file template rules. They catch markup that is difficult to
use with assistive technology or keyboard navigation.

## `a11y/img-alt`

Requires an `alt` attribute on `<img>`.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

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

## `a11y/alt-text`

Requires alternative text for media elements that need a text alternative.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

Good:

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit" />
</template>
```

## `a11y/click-events-have-key-events`

Reports click handlers on non-native interactive elements when no keyboard handler is present.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

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

## `a11y/interactive-supports-focus`

Requires elements with interactive roles to be focusable.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

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

Requires labels to be associated with a form control.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

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

## `a11y/form-control-has-label`

Requires controls to have a visible or programmatic label.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

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

## `a11y/no-aria-hidden-on-focusable`

Reports focusable elements hidden from assistive technology.

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

Good:

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-static-element-interactions`

Reports mouse or keyboard handlers on static elements.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <section @click="select">Select</section>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="select">Select</button>
</template>
```

## `a11y/tabindex-no-positive`

Reports positive `tabindex` values because they create a custom tab order that is hard to predict.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

Good:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/anchor-is-valid`

Requires anchors to have valid link targets.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <a href="#" @click="open">Open</a>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## Additional Accessibility Rules

`a11y/anchor-has-content` requires anchor elements to have accessible content. Default: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/aria-props` disallows invalid ARIA attributes. Default: `error`. Presets: `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-role` requires valid, non-abstract ARIA roles. Default: `error`. Presets: `happy-path`,
`nuxt`, `opinionated`.

`a11y/aria-unsupported-elements` disallows ARIA attributes on elements that do not support them.
Default: `error`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-has-content` requires heading elements to have accessible content. Default: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/heading-levels` disallows skipped heading levels. Default: `warning`. Presets: `nuxt`,
`opinionated`.

`a11y/iframe-has-title` requires `<iframe>` to have a `title`. Default: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/landmark-roles` validates landmark role placement and uniqueness. Default: `warning`.
Presets: `nuxt`, `opinionated`.

`a11y/media-has-caption` requires captions for media elements. Default: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/mouse-events-have-key-events` requires focus and blur handlers when mouse handlers are used.
Default: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-access-key` disallows the `accesskey` attribute. Default: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-autofocus` disallows `autofocus`. Default: `warning`. Presets: `happy-path`, `nuxt`,
`opinionated`.

`a11y/no-distracting-elements` disallows distracting elements such as `<marquee>` and `<blink>`.
Default: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-i-for-icon` discourages using `<i>` as an icon-only element. Default: `warning`. Presets:
`happy-path`, `nuxt`, `opinionated`.

`a11y/no-redundant-roles` disallows ARIA roles that duplicate native semantics. Default:
`warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-refer-to-non-existent-id` reports ARIA references to missing IDs. Default: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/no-role-presentation-on-focusable` disallows `role="presentation"` or `role="none"` on
focusable elements. Default: `error`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/placeholder-label-option` requires disabled or hidden on placeholder `<option>` values.
Default: `warning`. Presets: `nuxt`, `opinionated`.

`a11y/role-has-required-aria-props` requires roles to include their required ARIA attributes.
Default: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`a11y/use-list` suggests list elements for bullet-like text. Default: `warning`. Presets: `nuxt`,
`opinionated`.
