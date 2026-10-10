---
title: "Accessibility structure"
---

# Accessibility structure

Use these rules to review how a page is organized and described: headings, landmarks, labels,
frames, media, and focusable interactive elements. Start here when a component adds a new section,
embedded frame, image, or media player.

For example, [heading-levels](./reference/a11y-heading-levels.md) provides a concrete heading-order
check. Compare its Bad/Good examples, enable the rule using its configuration, and run `vp run lint`.
For the full set, follow the [accessibility guide](./accessibility.md).

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/heading-has-content`](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-has-content) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-has-content-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-has-content-good) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-levels) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-levels-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-heading-levels-good) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](https://vizejs.dev/rules/accessibility-structure.html#a11y-iframe-has-title) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-iframe-has-title-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-iframe-has-title-good) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](https://vizejs.dev/rules/accessibility-structure.html#a11y-img-alt) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-img-alt-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](https://vizejs.dev/rules/accessibility-structure.html#a11y-interactive-supports-focus) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-interactive-supports-focus-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-interactive-supports-focus-good) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](https://vizejs.dev/rules/accessibility-structure.html#a11y-label-has-for) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-label-has-for-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-label-has-for-good) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](https://vizejs.dev/rules/accessibility-structure.html#a11y-landmark-roles) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-landmark-roles-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-landmark-roles-good) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](https://vizejs.dev/rules/accessibility-structure.html#a11y-media-has-caption) | [Bad](https://vizejs.dev/rules/accessibility-structure.html#a11y-media-has-caption-bad) · [Good](https://vizejs.dev/rules/accessibility-structure.html#a11y-media-has-caption-good) | Require media elements to have captions |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
