---
title: Accessibility interactions
---

# Accessibility interactions

Use these rules when reviewing how people interact with a component: keyboard access, focus,
mouse handlers, hidden controls, and references to other elements. Check them when introducing
custom controls or changing visibility and event handling.

For example, a focusable element hidden from assistive technology is covered by
[no-aria-hidden-on-focusable](./reference/a11y-no-aria-hidden-on-focusable.md).
Compare its Bad/Good examples, use the shown configuration, then run `vp run lint`.
The [complete accessibility guide](./accessibility.md) explains how these checks fit together.

- [`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md)
- [`a11y/no-access-key`](./reference/a11y-no-access-key.md)
- [`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md)
- [`a11y/no-autofocus`](./reference/a11y-no-autofocus.md)
- [`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md)
- [`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md)
- [`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md)
- [`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md)

[All accessibility rules](./accessibility.md)
