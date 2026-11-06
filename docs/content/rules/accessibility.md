---
title: "Accessibility rules"
description: "ARIA, keyboard interaction, labels, landmarks and media alternatives, so templates work with assistive technology."
---

# Accessibility rules

ARIA, keyboard interaction, labels, landmarks and media alternatives, so templates work with assistive technology.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Default (25)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Require alternative text for media elements. |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Require anchor elements to have accessible content. |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Enforce valid href on anchor elements. |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Disallow invalid ARIA attributes. |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Elements with ARIA roles must use a valid, non-abstract ARIA role. |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Disallow ARIA attributes on elements that do not support them. |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Require keyboard event handlers with click events. |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Require form controls to have associated labels. |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Require heading elements to have accessible content. |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Require iframe elements to have a title attribute. |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Require interactive role elements to be focusable. |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Require labels to have associated form controls. |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Require media elements to have captions. |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Require focus/blur events with mouse events. |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Disallow the use of the accesskey attribute. |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Disallow aria-hidden="true" on focusable elements. |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Disallow the use of the autofocus attribute. |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;. |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Disallow using &lt;i&gt; element for icons. |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Disallow redundant ARIA roles. | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Disallow references to non-existent IDs. |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Disallow role="presentation" or role="none" on focusable elements. |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Disallow event handlers on static elements. |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Require ARIA roles to have required properties. |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Disallow positive tabindex values. |  |

## Stricter presets (5)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Disallow skipping heading levels. | `nuxt`, `opinionated` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Validate landmark role placement and uniqueness. | `nuxt`, `opinionated` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Require a placeholder label option in a required select. | `nuxt`, `opinionated` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Suggest using list elements for bullet-like text. | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Enforce unique element IDs using useId() instead of static literals. | `nuxt`, `opinionated` |  |

## Opt-in (1)

Not in any preset. Turn them on by name in the `linter.rules` passed to `withVue`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Require alt attribute on images for accessibility. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
