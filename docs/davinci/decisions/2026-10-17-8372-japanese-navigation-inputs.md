# Japanese navigation and generated hub inputs

Refs [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).
This follows the bounded Japanese performance reading-path change. It does not
complete the site-wide editorial review.

The previous Japanese navigation had 91 explicit labels and deliberately fell back to
English for 14 blog entries. All 14 destinations already have reviewed Japanese
page titles. Use those existing titles as the Japanese labels, preserving every
route and article body. Brand names and the package name remain unchanged.

The generated composable hub's introduction used the literal wording
"全エントリー: ランタイム契約" and left "ターゲット" unexplained. Explain what
readers can look up: entry-point APIs, SSR/hydration behavior, resource cleanup and
supported runtime environments. Capability targets refer to web, server, worker,
native, desktop and terminal environments in the existing metadata; this is not
a new compatibility or acceptance claim. Individual API descriptions remain
English. The existing device/permission and untested-preview warnings remain.

The Japanese UI hub linked its setup step to the English Vite+ guide even though
the Japanese guide exists. Select that existing Japanese route and remove the
English label. The copy and link are changed in their owning generator inputs,
so generation cannot restore the previous wording. Other locales and all example
code remain unchanged.

Tests exercise the real generator and compare all 14 labels with their authored
page headings. Source checks, exact Actions, complete rendered hub/navigation
review, merge and deployed reading-flow verification remain separate delivery
requirements. No publication or whole-site acceptance is claimed here.
