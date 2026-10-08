---
title: Accessibility rules
---

# Accessibility rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`a11y/alt-text`](./reference/a11y-alt-text.md) | Require alternative text for media elements |
| [`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Require anchor elements to have accessible content |
| [`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Enforce valid href on anchor elements |
| [`a11y/aria-props`](./reference/a11y-aria-props.md) | Disallow invalid ARIA attributes |
| [`a11y/aria-role`](./reference/a11y-aria-role.md) | Elements with ARIA roles must use a valid, non-abstract ARIA role |
| [`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Disallow ARIA attributes on elements that do not support them |
| [`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Require keyboard event handlers with click events |
| [`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Require form controls to have associated labels |
| [`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](./reference/a11y-img-alt.md) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Require media elements to have captions |
| [`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Disallow references to non-existent IDs |
| [`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Disallow positive tabindex values |
| [`a11y/use-list`](./reference/a11y-use-list.md) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
