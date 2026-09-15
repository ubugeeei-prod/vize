---
title: Accessibility Rules: Integrity
---

# Accessibility Rules: Integrity

Integrity checks catch focusable presentation roles, static interaction handlers, placeholder-only labels, incomplete ARIA roles, tab order traps, list semantics, and unstable IDs. These pages document every accessibility rule at the same level; there is no
lower-priority "additional" tier, and every rule includes options plus Bad/Good examples.

## `a11y/no-role-presentation-on-focusable`

Disallows `role="presentation"` or `role="none"` on focusable elements.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

Good:

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

## `a11y/no-static-element-interactions`

Disallows event handlers on static elements that do not expose matching interactive semantics.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

Good:

```vue
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

## `a11y/placeholder-label-option`

Requires placeholder `<option>` entries to be disabled or hidden so placeholder choices are not
submitted as real selections.

Default severity: `warning`
Presets: `nuxt`, `opinionated`
Options: none

Bad:

```vue
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Good:

```vue
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## `a11y/role-has-required-aria-props`

Requires ARIA roles to include the state or property attributes that the role needs.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

Bad:

```vue
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

Good:

```vue
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

## `a11y/tabindex-no-positive`

Disallows positive `tabindex` values because they create a custom tab order that is hard to predict
and maintain.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Options: none

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

## `a11y/use-list`

Suggests list elements for bullet-like text so screen readers can announce list structure.

Default severity: `warning`
Presets: `nuxt`, `opinionated`
Options: none

Bad:

```vue
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

Good:

```vue
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

## `vue/use-unique-element-ids`

Requires static IDs and ID references to use `useId()` instead of literals. This avoids duplicate
IDs when the same component renders multiple times and keeps label/ARIA relationships stable.

Default severity: `warning`
Presets: `nuxt`, `opinionated`
Options: none

Bad:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Good:

```vue
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```
