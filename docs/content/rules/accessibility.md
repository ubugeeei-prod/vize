---
title: Accessibility Rules
---

# Accessibility Rules

Accessibility rules are Patina single-file template rules. They flag markup that is hard to use
with assistive technology, keyboard navigation, stable element relationships, or accessible media.

Every rule below is documented at the same level. Preset membership only says where the rule is
enabled by default; it does not create a lower-priority "additional" tier. Current accessibility
rules do not accept `linter.ruleOptions`, so tune severity with `linter.rules`.

```json
{
  "linter": {
    "rules": {
      "a11y/img-alt": "error",
      "a11y/label-has-for": "warn",
      "vue/use-unique-element-ids": "warn"
    }
  }
}
```

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
