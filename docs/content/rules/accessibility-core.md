---
title: "Accessibility core"
---

# Accessibility core

Use these rules to check the basic accessibility information in a Vue template: image alternatives,
link text, valid ARIA attributes, keyboard equivalents, and form labels. Start here when adding
images, links, or inputs to a component.

For example, open [form-control-has-label](./reference/a11y-form-control-has-label.md) before adding
an input: compare its Bad/Good examples, enable the rule with the shown Vite+ configuration, and
run `vp run lint`. The [complete accessibility guide](./accessibility.md) explains the broader rule set.

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/alt-text`](https://vizejs.dev/rules/accessibility-core.html#a11y-alt-text) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-alt-text-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-alt-text-good) | Require alternative text for media elements |
| [`a11y/anchor-has-content`](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-has-content) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-has-content-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-has-content-good) | Require anchor elements to have accessible content |
| [`a11y/anchor-is-valid`](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-is-valid) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-is-valid-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-anchor-is-valid-good) | Enforce valid href on anchor elements |
| [`a11y/aria-props`](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-props) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-props-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-props-good) | Disallow invalid ARIA attributes |
| [`a11y/aria-role`](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-role) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-role-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-role-good) | Elements with ARIA roles must use a valid, non-abstract ARIA role |
| [`a11y/aria-unsupported-elements`](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-unsupported-elements) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-unsupported-elements-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-aria-unsupported-elements-good) | Disallow ARIA attributes on elements that do not support them |
| [`a11y/click-events-have-key-events`](https://vizejs.dev/rules/accessibility-core.html#a11y-click-events-have-key-events) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-click-events-have-key-events-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-click-events-have-key-events-good) | Require keyboard event handlers with click events |
| [`a11y/form-control-has-label`](https://vizejs.dev/rules/accessibility-core.html#a11y-form-control-has-label) | [Bad](https://vizejs.dev/rules/accessibility-core.html#a11y-form-control-has-label-bad) · [Good](https://vizejs.dev/rules/accessibility-core.html#a11y-form-control-has-label-good) | Require form controls to have associated labels |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
