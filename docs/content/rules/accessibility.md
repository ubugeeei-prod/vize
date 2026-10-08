---
title: Accessibility rules
---

# Accessibility rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/alt-text`](./all.md#a11y-alt-text) | [Bad](./all.md#a11y-alt-text-bad) · [Good](./all.md#a11y-alt-text-good) | Require alternative text for media elements |
| [`a11y/anchor-has-content`](./all.md#a11y-anchor-has-content) | [Bad](./all.md#a11y-anchor-has-content-bad) · [Good](./all.md#a11y-anchor-has-content-good) | Require anchor elements to have accessible content |
| [`a11y/anchor-is-valid`](./all.md#a11y-anchor-is-valid) | [Bad](./all.md#a11y-anchor-is-valid-bad) · [Good](./all.md#a11y-anchor-is-valid-good) | Enforce valid href on anchor elements |
| [`a11y/aria-props`](./all.md#a11y-aria-props) | [Bad](./all.md#a11y-aria-props-bad) · [Good](./all.md#a11y-aria-props-good) | Disallow invalid ARIA attributes |
| [`a11y/aria-role`](./all.md#a11y-aria-role) | [Bad](./all.md#a11y-aria-role-bad) · [Good](./all.md#a11y-aria-role-good) | Elements with ARIA roles must use a valid, non-abstract ARIA role |
| [`a11y/aria-unsupported-elements`](./all.md#a11y-aria-unsupported-elements) | [Bad](./all.md#a11y-aria-unsupported-elements-bad) · [Good](./all.md#a11y-aria-unsupported-elements-good) | Disallow ARIA attributes on elements that do not support them |
| [`a11y/click-events-have-key-events`](./all.md#a11y-click-events-have-key-events) | [Bad](./all.md#a11y-click-events-have-key-events-bad) · [Good](./all.md#a11y-click-events-have-key-events-good) | Require keyboard event handlers with click events |
| [`a11y/form-control-has-label`](./all.md#a11y-form-control-has-label) | [Bad](./all.md#a11y-form-control-has-label-bad) · [Good](./all.md#a11y-form-control-has-label-good) | Require form controls to have associated labels |
| [`a11y/heading-has-content`](./all.md#a11y-heading-has-content) | [Bad](./all.md#a11y-heading-has-content-bad) · [Good](./all.md#a11y-heading-has-content-good) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](./all.md#a11y-heading-levels) | [Bad](./all.md#a11y-heading-levels-bad) · [Good](./all.md#a11y-heading-levels-good) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](./all.md#a11y-iframe-has-title) | [Bad](./all.md#a11y-iframe-has-title-bad) · [Good](./all.md#a11y-iframe-has-title-good) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](./all.md#a11y-img-alt) | [Bad](./all.md#a11y-img-alt-bad) · [Good](./all.md#a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](./all.md#a11y-interactive-supports-focus) | [Bad](./all.md#a11y-interactive-supports-focus-bad) · [Good](./all.md#a11y-interactive-supports-focus-good) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](./all.md#a11y-label-has-for) | [Bad](./all.md#a11y-label-has-for-bad) · [Good](./all.md#a11y-label-has-for-good) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](./all.md#a11y-landmark-roles) | [Bad](./all.md#a11y-landmark-roles-bad) · [Good](./all.md#a11y-landmark-roles-good) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](./all.md#a11y-media-has-caption) | [Bad](./all.md#a11y-media-has-caption-bad) · [Good](./all.md#a11y-media-has-caption-good) | Require media elements to have captions |
| [`a11y/mouse-events-have-key-events`](./all.md#a11y-mouse-events-have-key-events) | [Bad](./all.md#a11y-mouse-events-have-key-events-bad) · [Good](./all.md#a11y-mouse-events-have-key-events-good) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](./all.md#a11y-no-access-key) | [Bad](./all.md#a11y-no-access-key-bad) · [Good](./all.md#a11y-no-access-key-good) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](./all.md#a11y-no-aria-hidden-on-focusable) | [Bad](./all.md#a11y-no-aria-hidden-on-focusable-bad) · [Good](./all.md#a11y-no-aria-hidden-on-focusable-good) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](./all.md#a11y-no-autofocus) | [Bad](./all.md#a11y-no-autofocus-bad) · [Good](./all.md#a11y-no-autofocus-good) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](./all.md#a11y-no-distracting-elements) | [Bad](./all.md#a11y-no-distracting-elements-bad) · [Good](./all.md#a11y-no-distracting-elements-good) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](./all.md#a11y-no-i-for-icon) | [Bad](./all.md#a11y-no-i-for-icon-bad) · [Good](./all.md#a11y-no-i-for-icon-good) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](./all.md#a11y-no-redundant-roles) | [Bad](./all.md#a11y-no-redundant-roles-bad) · [Good](./all.md#a11y-no-redundant-roles-good) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](./all.md#a11y-no-refer-to-non-existent-id) | [Bad](./all.md#a11y-no-refer-to-non-existent-id-bad) · [Good](./all.md#a11y-no-refer-to-non-existent-id-good) | Disallow references to non-existent IDs |
| [`a11y/no-role-presentation-on-focusable`](./all.md#a11y-no-role-presentation-on-focusable) | [Bad](./all.md#a11y-no-role-presentation-on-focusable-bad) · [Good](./all.md#a11y-no-role-presentation-on-focusable-good) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](./all.md#a11y-no-static-element-interactions) | [Bad](./all.md#a11y-no-static-element-interactions-bad) · [Good](./all.md#a11y-no-static-element-interactions-good) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](./all.md#a11y-placeholder-label-option) | [Bad](./all.md#a11y-placeholder-label-option-bad) · [Good](./all.md#a11y-placeholder-label-option-good) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](./all.md#a11y-role-has-required-aria-props) | [Bad](./all.md#a11y-role-has-required-aria-props-bad) · [Good](./all.md#a11y-role-has-required-aria-props-good) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](./all.md#a11y-tabindex-no-positive) | [Bad](./all.md#a11y-tabindex-no-positive-bad) · [Good](./all.md#a11y-tabindex-no-positive-good) | Disallow positive tabindex values |
| [`a11y/use-list`](./all.md#a11y-use-list) | [Bad](./all.md#a11y-use-list-bad) · [Good](./all.md#a11y-use-list-good) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](./all.md#vue-use-unique-element-ids) | [Bad](./all.md#vue-use-unique-element-ids-bad) · [Good](./all.md#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
