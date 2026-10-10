---
title: "Accessibility integrity"
---

# Accessibility integrity

Use these rules when a template's roles, focus order, labels, or element relationships disagree.
They help review the consistency of markup after adding ARIA roles or changing an interactive component.

For example, [role-has-required-aria-props](./reference/a11y-role-has-required-aria-props.md) shows
which change to make when an assigned role is missing a required property. Compare the examples,
use that rule's configuration, then run `vp run lint`.
The [complete accessibility guide](./accessibility.md) links the other checks.

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/no-role-presentation-on-focusable`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-role-presentation-on-focusable-good) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-static-element-interactions) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-static-element-interactions-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-no-static-element-interactions-good) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-placeholder-label-option) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-placeholder-label-option-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-placeholder-label-option-good) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-role-has-required-aria-props) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-role-has-required-aria-props-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-role-has-required-aria-props-good) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-tabindex-no-positive) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-tabindex-no-positive-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-tabindex-no-positive-good) | Disallow positive tabindex values |
| [`a11y/use-list`](https://vizejs.dev/rules/accessibility-integrity.html#a11y-use-list) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#a11y-use-list-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#a11y-use-list-good) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](https://vizejs.dev/rules/accessibility-integrity.html#vue-use-unique-element-ids) | [Bad](https://vizejs.dev/rules/accessibility-integrity.html#vue-use-unique-element-ids-bad) · [Good](https://vizejs.dev/rules/accessibility-integrity.html#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
