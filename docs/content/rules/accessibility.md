---
title: Accessibility rules
---

# Accessibility rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/alt-text`](./reference/a11y-alt-text.md) | [Bad](./reference/a11y-alt-text.md#bad) · [Good](./reference/a11y-alt-text.md#good) | Require alternative text for media elements |
| [`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | [Bad](./reference/a11y-anchor-has-content.md#bad) · [Good](./reference/a11y-anchor-has-content.md#good) | Require anchor elements to have accessible content |
| [`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | [Bad](./reference/a11y-anchor-is-valid.md#bad) · [Good](./reference/a11y-anchor-is-valid.md#good) | Enforce valid href on anchor elements |
| [`a11y/aria-props`](./reference/a11y-aria-props.md) | [Bad](./reference/a11y-aria-props.md#bad) · [Good](./reference/a11y-aria-props.md#good) | Disallow invalid ARIA attributes |
| [`a11y/aria-role`](./reference/a11y-aria-role.md) | [Bad](./reference/a11y-aria-role.md#bad) · [Good](./reference/a11y-aria-role.md#good) | Elements with ARIA roles must use a valid, non-abstract ARIA role |
| [`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | [Bad](./reference/a11y-aria-unsupported-elements.md#bad) · [Good](./reference/a11y-aria-unsupported-elements.md#good) | Disallow ARIA attributes on elements that do not support them |
| [`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | [Bad](./reference/a11y-click-events-have-key-events.md#bad) · [Good](./reference/a11y-click-events-have-key-events.md#good) | Require keyboard event handlers with click events |
| [`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | [Bad](./reference/a11y-form-control-has-label.md#bad) · [Good](./reference/a11y-form-control-has-label.md#good) | Require form controls to have associated labels |
| [`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | [Bad](./reference/a11y-heading-has-content.md#bad) · [Good](./reference/a11y-heading-has-content.md#good) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](./reference/a11y-heading-levels.md) | [Bad](./reference/a11y-heading-levels.md#bad) · [Good](./reference/a11y-heading-levels.md#good) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | [Bad](./reference/a11y-iframe-has-title.md#bad) · [Good](./reference/a11y-iframe-has-title.md#good) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](./reference/a11y-img-alt.md) | [Bad](./reference/a11y-img-alt.md#bad) · [Good](./reference/a11y-img-alt.md#good) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | [Bad](./reference/a11y-interactive-supports-focus.md#bad) · [Good](./reference/a11y-interactive-supports-focus.md#good) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](./reference/a11y-label-has-for.md) | [Bad](./reference/a11y-label-has-for.md#bad) · [Good](./reference/a11y-label-has-for.md#good) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | [Bad](./reference/a11y-landmark-roles.md#bad) · [Good](./reference/a11y-landmark-roles.md#good) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | [Bad](./reference/a11y-media-has-caption.md#bad) · [Good](./reference/a11y-media-has-caption.md#good) | Require media elements to have captions |
| [`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | [Bad](./reference/a11y-mouse-events-have-key-events.md#bad) · [Good](./reference/a11y-mouse-events-have-key-events.md#good) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](./reference/a11y-no-access-key.md) | [Bad](./reference/a11y-no-access-key.md#bad) · [Good](./reference/a11y-no-access-key.md#good) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | [Bad](./reference/a11y-no-aria-hidden-on-focusable.md#bad) · [Good](./reference/a11y-no-aria-hidden-on-focusable.md#good) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | [Bad](./reference/a11y-no-autofocus.md#bad) · [Good](./reference/a11y-no-autofocus.md#good) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | [Bad](./reference/a11y-no-distracting-elements.md#bad) · [Good](./reference/a11y-no-distracting-elements.md#good) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | [Bad](./reference/a11y-no-i-for-icon.md#bad) · [Good](./reference/a11y-no-i-for-icon.md#good) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | [Bad](./reference/a11y-no-redundant-roles.md#bad) · [Good](./reference/a11y-no-redundant-roles.md#good) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | [Bad](./reference/a11y-no-refer-to-non-existent-id.md#bad) · [Good](./reference/a11y-no-refer-to-non-existent-id.md#good) | Disallow references to non-existent IDs |
| [`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | [Bad](./reference/a11y-no-role-presentation-on-focusable.md#bad) · [Good](./reference/a11y-no-role-presentation-on-focusable.md#good) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | [Bad](./reference/a11y-no-static-element-interactions.md#bad) · [Good](./reference/a11y-no-static-element-interactions.md#good) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | [Bad](./reference/a11y-placeholder-label-option.md#bad) · [Good](./reference/a11y-placeholder-label-option.md#good) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | [Bad](./reference/a11y-role-has-required-aria-props.md#bad) · [Good](./reference/a11y-role-has-required-aria-props.md#good) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | [Bad](./reference/a11y-tabindex-no-positive.md#bad) · [Good](./reference/a11y-tabindex-no-positive.md#good) | Disallow positive tabindex values |
| [`a11y/use-list`](./reference/a11y-use-list.md) | [Bad](./reference/a11y-use-list.md#bad) · [Good](./reference/a11y-use-list.md#good) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | [Bad](./reference/vue-use-unique-element-ids.md#bad) · [Good](./reference/vue-use-unique-element-ids.md#good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
