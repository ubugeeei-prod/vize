---
title: Accessibility integrity
---

# Accessibility integrity

Use these rules when a template's roles, focus order, labels, or element relationships disagree.
They help review the consistency of markup after adding ARIA roles or changing an interactive component.

For example, [role-has-required-aria-props](./reference/a11y-role-has-required-aria-props.md) shows
which change to make when an assigned role is missing a required property. Compare the examples,
use that rule's configuration, then run `vp run lint`.
The [complete accessibility guide](./accessibility.md) links the other checks.

- [`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md)
- [`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md)
- [`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md)
- [`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md)
- [`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md)
- [`a11y/use-list`](./reference/a11y-use-list.md)
- [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md)

[All accessibility rules](./accessibility.md)
