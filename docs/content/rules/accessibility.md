---
title: Accessibility Rules
---

# Accessibility Rules

Accessibility rules are Patina single-file template rules. All rules below are documented at the
same level: preset membership changes where a rule is enabled by default, but there is no secondary
"additional" tier. Current accessibility rules take no `linter.ruleOptions`; configure severity with
`linter.rules`.
## `a11y/alt-text`
Requires text alternatives for media elements. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<input type="image" src="/submit.png" />
```
Good:
```vue
<input type="image" src="/submit.png" alt="Submit search" />
```

## `a11y/anchor-has-content`
Requires anchors to expose visible text, an accessible label, or labelled children. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<a href="/settings"></a>
```
Good:
```vue
<a href="/settings">Settings</a>
```

## `a11y/anchor-is-valid`
Requires anchors to have real link targets; static `href` values are normalized before checking. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<a href="#" @click="openPanel">Open panel</a>
```
Good:
```vue
<button type="button" @click="openPanel">Open panel</button>
```

## `a11y/aria-props`
Disallows invalid or misspelled ARIA attributes. Default severity: `error`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button aria-lable="Save changes">Save</button>
```
Good:
```vue
<button aria-label="Save changes">Save</button>
```

## `a11y/aria-role`
Requires ARIA roles to be valid concrete roles. Default severity: `error`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<section role="datepicker">...</section>
```
Good:
```vue
<section role="dialog" aria-label="Choose a date">...</section>
```

## `a11y/aria-unsupported-elements`
Disallows ARIA attributes on elements that do not support ARIA semantics. Default severity: `error`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<meta charset="utf-8" aria-hidden="true" />
```
Good:
```vue
<meta charset="utf-8" />
```

## `a11y/click-events-have-key-events`
Requires keyboard access when click handlers are used on non-native interactive elements. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<div role="button" @click="save">Save</div>
```
Good:
```vue
<button type="button" @click="save">Save</button>
```

## `a11y/form-control-has-label`
Requires form controls to have a visible label or programmatic accessible name. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<input type="search" />
```
Good:
```vue
<label>Search <input type="search" /></label>
```

## `a11y/heading-has-content`
Requires heading elements to have accessible content. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<h2></h2>
```
Good:
```vue
<h2>Billing settings</h2>
```

## `a11y/heading-levels`
Disallows skipped heading levels so the page outline stays predictable. Default severity: `warning`; Presets: `nuxt`, `opinionated`; Options: none.
Bad:
```vue
<h1>Account</h1><h3>Billing</h3>
```
Good:
```vue
<h1>Account</h1><h2>Billing</h2>
```

## `a11y/iframe-has-title`
Requires iframe elements to describe embedded content with `title`. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<iframe src="/checkout"></iframe>
```
Good:
```vue
<iframe src="/checkout" title="Checkout preview"></iframe>
```

## `a11y/img-alt`
Requires images to include `alt`; decorative images should use empty `alt`. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<img src="/avatar.png" />
```
Good:
```vue
<img src="/avatar.png" alt="User avatar" />
```

## `a11y/interactive-supports-focus`
Requires elements with interactive roles to be focusable. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<span role="button" @click="open">Open</span>
```
Good:
```vue
<button type="button" @click="open">Open</button>
```

## `a11y/label-has-for`
Requires labels to be associated with controls through `for`/`id` or wrapping. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<label>Email</label><input id="email" />
```
Good:
```vue
<label for="email">Email</label><input id="email" />
```

## `a11y/landmark-roles`
Validates landmark placement and uniqueness. Default severity: `warning`; Presets: `nuxt`, `opinionated`; Options: none.
Bad:
```vue
<main>Dashboard</main><main>Settings</main>
```
Good:
```vue
<main>Dashboard</main><nav aria-label="Settings">...</nav>
```

## `a11y/media-has-caption`
Requires captions or text tracks for audio and video with spoken content. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<video src="/demo.mp4" controls />
```
Good:
```vue
<video src="/demo.mp4" controls><track kind="captions" src="/demo.en.vtt" srclang="en" label="English" /></video>
```

## `a11y/mouse-events-have-key-events`
Requires focus and blur handlers when hover handlers are used. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
```
Good:
```vue
<button @focus="showPreview" @blur="hidePreview" @mouseenter="showPreview" @mouseleave="hidePreview">Preview</button>
```

## `a11y/no-access-key`
Disallows `accesskey` because shortcuts conflict across browsers and assistive technology. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button accesskey="s">Save</button>
```
Good:
```vue
<button>Save</button>
```

## `a11y/no-aria-hidden-on-focusable`
Disallows `aria-hidden="true"` on focusable elements. Default severity: `error`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button aria-hidden="true" @click="close">Close</button>
```
Good:
```vue
<button aria-label="Close" @click="close">Close</button>
```

## `a11y/no-autofocus`
Disallows `autofocus` because it can move focus unexpectedly. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<input autofocus name="query" />
```
Good:
```vue
<input name="query" />
```

## `a11y/no-distracting-elements`
Disallows distracting legacy elements such as `<marquee>` and `<blink>`. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<marquee>Limited offer</marquee>
```
Good:
```vue
<p>Limited offer</p>
```

## `a11y/no-i-for-icon`
Disallows using `<i>` as an icon-only element. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button><i class="material-icons">delete</i></button>
```
Good:
```vue
<button><span class="material-icons" aria-hidden="true">delete</span><span class="sr-only">Delete item</span></button>
```

## `a11y/no-redundant-roles`
Disallows ARIA roles that duplicate an element's implicit role. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button role="button">Save</button>
```
Good:
```vue
<button>Save</button>
```

## `a11y/no-refer-to-non-existent-id`
Requires ARIA ID references and label relationships to point at existing elements. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button aria-labelledby="save-label">Save</button>
```
Good:
```vue
<span id="save-label">Save changes</span><button aria-labelledby="save-label">Save</button>
```

## `a11y/no-role-presentation-on-focusable`
Disallows `role="presentation"` or `role="none"` on focusable elements. Default severity: `error`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<a href="/billing" role="presentation">Billing</a>
```
Good:
```vue
<a href="/billing">Billing</a>
```

## `a11y/no-static-element-interactions`
Disallows event handlers on static elements that do not expose matching interactive semantics. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<section @keydown.enter="select">Select</section>
```
Good:
```vue
<button type="button" @keydown.enter="select">Select</button>
```

## `a11y/placeholder-label-option`
Requires placeholder `<option>` entries to be disabled or hidden. Default severity: `warning`; Presets: `nuxt`, `opinionated`; Options: none.
Bad:
```vue
<select><option value="">Choose</option><option value="jp">Japan</option></select>
```
Good:
```vue
<select><option value="" disabled>Choose</option><option value="jp">Japan</option></select>
```

## `a11y/role-has-required-aria-props`
Requires ARIA roles to include their required state or property attributes. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<span role="checkbox">Receive updates</span>
```
Good:
```vue
<span role="checkbox" aria-checked="false">Receive updates</span>
```

## `a11y/tabindex-no-positive`
Disallows positive `tabindex` values that create a custom tab order. Default severity: `warning`; Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`; Options: none.
Bad:
```vue
<button tabindex="3">Save</button>
```
Good:
```vue
<button>Save</button>
```

## `a11y/use-list`
Suggests list elements for bullet-like text so list structure is announced. Default severity: `warning`; Presets: `nuxt`, `opinionated`; Options: none.
Bad:
```vue
<p>- First task</p><p>- Second task</p>
```
Good:
```vue
<ul><li>First task</li><li>Second task</li></ul>
```

## `vue/use-unique-element-ids`
Requires static IDs and ID references to use `useId()` instead of literals. Default severity: `warning`; Presets: `nuxt`, `opinionated`; Options: none.
Bad:
```vue
<label for="email">Email</label><input id="email" />
```
Good:
```vue
<script setup>import { useId } from "vue"; const emailId = useId();</script><template><label :for="emailId">Email</label><input :id="emailId" /></template>
```
