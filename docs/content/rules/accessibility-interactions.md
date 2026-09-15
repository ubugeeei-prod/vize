---
title: Accessibility Rules: Interactions
---

# Accessibility Rules: Interactions

Interaction checks prevent pointer-only affordances, hidden focus targets, forced focus jumps, and ambiguous decorative markup. These pages document every accessibility rule at the same level; there is no
lower-priority "additional" tier, and every rule includes options plus Bad/Good examples.

## `a11y/mouse-events-have-key-events`

Requires matching focus and blur handlers when hover handlers are used.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

Good:

```vue
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

## `a11y/no-access-key`

Disallows `accesskey` because custom shortcuts conflict across browsers, operating systems, and
assistive technology.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

Good:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

Disallows `aria-hidden="true"` on focusable elements. This prevents a control from disappearing
from the accessibility tree while still receiving keyboard focus.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

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

## `a11y/no-autofocus`

Disallows `autofocus` because it can move focus unexpectedly when a page or dialog appears.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <input autofocus name="query" />
</template>
```

Good:

```vue
<template>
  <input name="query" />
</template>
```

## `a11y/no-distracting-elements`

Disallows distracting legacy elements such as `<marquee>` and `<blink>`.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <marquee>Limited offer</marquee>
</template>
```

Good:

```vue
<template>
  <p>Limited offer</p>
</template>
```

## `a11y/no-i-for-icon`

Disallows using `<i>` as an icon-only element. Use a neutral element plus an accessible text path
when the icon has meaning.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

Good:

```vue
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

## `a11y/no-redundant-roles`

Disallows ARIA roles that duplicate an element's implicit role.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <button role="button">Save</button>
</template>
```

Good:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/no-refer-to-non-existent-id`

Requires ARIA ID references and label relationships to point at elements that exist in the same
template.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Good:

```vue
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```
