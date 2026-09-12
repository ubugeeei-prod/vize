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

## `a11y/anchor-has-content`

Requires anchor elements to have content that can become an accessible name. Text, an
interpolation, an element with accessible content, an image with non-empty `alt`, `aria-label`, and
`aria-labelledby` all satisfy the rule.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

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
  <a href="/settings" aria-label="Settings"></a>
</template>
```

## `a11y/aria-props`

Disallows invalid `aria-*` attributes. This catches typos and non-standard ARIA names before they
silently disappear from the accessibility tree.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <button aria-lable="Close">x</button>
</template>
```

Good:

```vue
<template>
  <button aria-label="Close">x</button>
</template>
```

## `a11y/aria-role`

Requires `role` values to be valid, concrete ARIA roles. Unknown roles and abstract roles are
reported because assistive technology cannot expose them as intended.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <div role="datepicker">Choose a date</div>
  <div role="command">Run</div>
</template>
```

Good:

```vue
<template>
  <button type="button">Choose a date</button>
  <div role="button" tabindex="0">Run</div>
</template>
```

## `a11y/aria-unsupported-elements`

Disallows ARIA attributes and `role` on elements that do not participate in the accessibility tree,
such as metadata and script/style elements.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <meta aria-hidden="true" />
  <script role="presentation"></script>
</template>
```

Good:

```vue
<template>
  <meta name="viewport" content="width=device-width" />
  <script></script>
</template>
```

## `a11y/heading-has-content`

Requires `<h1>` through `<h6>` to have accessible content. Visible text, interpolation, accessible
children, or an accessible name through ARIA satisfy the rule.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <h2></h2>
</template>
```

Good:

```vue
<template>
  <h2>Billing</h2>
  <h2 aria-label="Billing"></h2>
</template>
```

## `a11y/heading-levels`

Reports skipped heading levels. Heading order should describe the page outline, so a section should
not jump from `<h1>` to `<h3>` without an intervening `<h2>`.

Default severity: `warning`
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <h1>Account</h1>
  <h3>Invoices</h3>
</template>
```

Good:

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
  <h3>Invoices</h3>
</template>
```

## `a11y/iframe-has-title`

Requires every `<iframe>` to have a non-empty `title`, or a bound `title` when the value is dynamic.
Screen readers use the title to identify the embedded document.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <iframe src="/billing"></iframe>
</template>
```

Good:

```vue
<template>
  <iframe src="/billing" title="Billing dashboard"></iframe>
</template>
```

## `a11y/landmark-roles`

Validates landmark placement and uniqueness. It reports duplicate `main` landmarks, nested
landmarks that compete with each other, and repeated navigation or region landmarks without
distinct labels.

Default severity: `warning`
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <main>Primary content</main>
  <main>Secondary content</main>
  <nav>Primary navigation</nav>
  <nav>Footer navigation</nav>
</template>
```

Good:

```vue
<template>
  <main>Primary content</main>
  <nav aria-label="Primary">Primary navigation</nav>
  <nav aria-label="Footer">Footer navigation</nav>
</template>
```

## `a11y/media-has-caption`

Requires `<video>` and `<audio>` elements to provide captions through a
`<track kind="captions">` child. Muted media and explicitly labelled media are accepted when the
rule can prove the content does not need captions.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <video src="/launch.mp4"></video>
</template>
```

Good:

```vue
<template>
  <video src="/launch.mp4">
    <track kind="captions" src="/launch.en.vtt" />
  </video>
  <video src="/ambient.mp4" muted></video>
</template>
```

## `a11y/no-access-key`

Disallows `accesskey` on native elements. Browser, operating-system, and assistive-technology
shortcuts vary by platform, so custom access keys are easy to conflict with.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

Good:

```vue
<template>
  <button type="button">Save</button>
</template>
```

## `a11y/no-autofocus`

Disallows `autofocus`. Moving focus automatically can interrupt screen-reader reading order and
surprise keyboard users.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <input autofocus />
</template>
```

Good:

```vue
<template>
  <input />
</template>
```

## `a11y/no-distracting-elements`

Disallows elements such as `<marquee>` and `<blink>`. These tags create hard-to-control movement or
flashing and are poor fits for accessible, modern interfaces.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <marquee>Sale ends soon</marquee>
  <blink>Unread</blink>
</template>
```

Good:

```vue
<template>
  <p>Sale ends soon</p>
  <strong>Unread</strong>
</template>
```

## `a11y/no-redundant-roles`

Disallows explicit ARIA roles that duplicate native HTML semantics. The rule is fixable for static
redundant roles because removing the role keeps the same exposed meaning with less noise.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
Fixable: yes

Bad:

```vue
<template>
  <button role="button">Save</button>
  <nav role="navigation">Sections</nav>
</template>
```

Good:

```vue
<template>
  <button type="button">Save</button>
  <nav>Sections</nav>
  <div role="navigation">Supplemental links</div>
</template>
```

## `a11y/no-role-presentation-on-focusable`

Disallows `role="presentation"` and `role="none"` on focusable elements. A focusable element still
receives keyboard focus, so removing its semantics makes the focused control confusing.

Default severity: `error`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <button role="presentation">Close</button>
  <div role="none" tabindex="0">Focusable panel</div>
</template>
```

Good:

```vue
<template>
  <button type="button">Close</button>
  <div tabindex="0">Focusable panel</div>
</template>
```

## `a11y/placeholder-label-option`

Requires the placeholder option in a `<select>` to be disabled or hidden. The first empty-value
option is treated as the placeholder, and it should not remain a valid submitted choice.

Default severity: `warning`
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <select>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Good:

```vue
<template>
  <select>
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## `a11y/role-has-required-aria-props`

Requires ARIA roles to include their required ARIA properties. For example, checkboxes need
`aria-checked`, and sliders need value bounds plus the current value.

Default severity: `warning`
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`

Bad:

```vue
<template>
  <div role="checkbox">Subscribe</div>
  <div role="slider">Volume</div>
</template>
```

Good:

```vue
<template>
  <div role="checkbox" aria-checked="false">Subscribe</div>
  <div role="slider" aria-valuemin="0" aria-valuemax="100" aria-valuenow="50">Volume</div>
</template>
```

## `a11y/use-list`

Suggests semantic list markup when text content looks like a bullet list. Real lists give assistive
technology item counts, boundaries, and list navigation.

Default severity: `warning`
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p>- Create an account</p>
  <p>- Invite the team</p>
</template>
```

Good:

```vue
<template>
  <ul>
    <li>Create an account</li>
    <li>Invite the team</li>
  </ul>
</template>
```
