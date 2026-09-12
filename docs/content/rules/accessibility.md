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
Static `href` values are checked after scheme normalization, so `JaVaScRiPt:` and HTML-decoded
control characters inside `java&#x0A;script:` are still reported while similar non-matching schemes
stay allowed.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <a href="#" @click="open">Open</a>
  <a href="JaVaScRiPt:void(0)">Open</a>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="open">Open</button>
  <a href="/docs/javascript:void">Docs</a>
</template>
```

## `a11y/mouse-events-have-key-events`

Requires focus and blur handlers when mouse hover handlers are used.

Default severity: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

```vue
<!-- Bad -->
<div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>

<!-- Good -->
<div tabindex="0" @mouseenter="showPreview" @mouseleave="hidePreview" @focus="showPreview" @blur="hidePreview">Preview</div>
```

## `a11y/no-i-for-icon`

Reports `<i>` elements that are used only as icons through common icon CSS classes.

Default severity: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

```vue
<!-- Bad -->
<i class="material-icons">home</i>

<!-- Good -->
<span class="material-icons" aria-hidden="true">home</span>
<span class="sr-only">Home</span>
```

## `a11y/no-refer-to-non-existent-id`

Requires static ID references such as `for`, `aria-labelledby`, and `aria-describedby` to point at IDs that exist in the same template.

Default severity: `warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

```vue
<!-- Bad -->
<label for="email">Email</label>
<input id="user-email" />

<!-- Good -->
<label for="email">Email</label>
<input id="email" />
```

## Additional Accessibility Rules

The split here is documentation detail only: these rules run through the same Patina template
pipeline, support the same config surface, and report the same severity regardless of whether this
page gives them a full example section or a compact rule note.

| Rule | Default | What it checks |
| --- | --- | --- |
| `a11y/anchor-has-content` | `warning` | Anchors need an accessible name from text, interpolation, labelled children, non-empty image `alt`, `aria-label`, or `aria-labelledby`. |
| `a11y/aria-props` | `error` | Only valid `aria-*` attributes are allowed, so typos such as `aria-lable` do not silently disappear from the accessibility tree. |
| `a11y/aria-role` | `error` | `role` values must be concrete WAI-ARIA roles; unknown and abstract roles are rejected. |
| `a11y/aria-unsupported-elements` | `error` | Elements that are not exposed to assistive technology, such as metadata or script/style elements, must not carry ARIA attributes or roles. |
| `a11y/heading-has-content` | `warning` | `h1`-`h6` elements need visible text, interpolation, accessible children, or an ARIA name. |
| `a11y/heading-levels` | `warning` | Heading levels should not skip outline steps, for example from `h1` directly to `h3`. |
| `a11y/iframe-has-title` | `warning` | Each `iframe` needs a non-empty static title or a dynamic title binding. |
| `a11y/landmark-roles` | `warning` | Landmarks such as `main`, `nav`, and `region` must avoid duplicate or ambiguous page structure; repeated navigation/region landmarks need distinct labels. |
| `a11y/media-has-caption` | `warning` | `video` and `audio` need captions via `track kind="captions"` unless the rule can prove captions are unnecessary, such as muted ambient media. |
| `a11y/no-access-key` | `warning` | Native elements should not use `accesskey`, which collides with browser, OS, and assistive-technology shortcuts. |
| `a11y/no-autofocus` | `warning` | `autofocus` is disallowed because automatic focus movement interrupts reading order and keyboard flow. |
| `a11y/no-distracting-elements` | `warning` | Deprecated moving or flashing elements such as `marquee` and `blink` are rejected. |
| `a11y/no-redundant-roles` | `warning` | Explicit roles that duplicate native semantics, such as `button role="button"`, are reported and can be fixed by removing the role. |
| `a11y/no-role-presentation-on-focusable` | `error` | Focusable elements must not use `role="presentation"` or `role="none"`, because focus remains while semantics disappear. |
| `a11y/placeholder-label-option` | `warning` | The first empty-value placeholder option in a `select` should be `disabled` or `hidden` so it cannot be submitted as a real choice. |
| `a11y/role-has-required-aria-props` | `warning` | Roles that require ARIA state, such as checkbox or slider, must include the required properties like `aria-checked` or value bounds. |
| `a11y/use-list` | `warning` | Text that looks like a bullet list should use semantic `ul`/`ol` and `li` markup so assistive technology can expose list boundaries and item counts. |
