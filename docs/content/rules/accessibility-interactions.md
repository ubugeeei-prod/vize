---
title: "Accessibility interactions"
---

# Accessibility interactions

Use these rules when reviewing how people interact with a component: keyboard access, focus,
mouse handlers, hidden controls, and references to other elements. Check them when introducing
custom controls or changing visibility and event handling.

For example, a focusable element hidden from assistive technology is covered by
[no-aria-hidden-on-focusable](./reference/a11y-no-aria-hidden-on-focusable.md).
Compare its Bad/Good examples, use the shown configuration, then run `vp run lint`.
The [complete accessibility guide](./accessibility.md) explains how these checks fit together.

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/mouse-events-have-key-events`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-mouse-events-have-key-events-good) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-access-key) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-access-key-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-access-key-good) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-aria-hidden-on-focusable-good) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-autofocus) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-autofocus-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-autofocus-good) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-distracting-elements) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-distracting-elements-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-distracting-elements-good) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-i-for-icon) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-i-for-icon-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-i-for-icon-good) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-redundant-roles) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-redundant-roles-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-redundant-roles-good) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id) | [Bad](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id-bad) · [Good](https://vizejs.dev/rules/accessibility-interactions.html#a11y-no-refer-to-non-existent-id-good) | Disallow references to non-existent IDs |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
