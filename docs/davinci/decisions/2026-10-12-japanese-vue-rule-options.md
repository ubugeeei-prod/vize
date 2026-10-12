# Japanese Vue rule options

This reader slice addresses [#8372](https://github.com/ubugeeei-prod/vize/issues/8372)
and the `/rules/options-vue` entry in
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

The new complete Japanese article explains when to adjust each Vue rule:
slot modifiers, component naming, prop mutation, SFC block order, self-closing
syntax, event names and attribute names. Its practical reference link stays
localized, and explicit aliases preserve all eight existing English bookmarks.
The twenty whole original examples, ordered API tokens and link destination
remain unchanged. Defaults follow the existing typed rule options; this
documentation change adds no option, preset or production behavior.

One existing English sentence needed a precise correction. For
`vue/html-self-closing`, `never` forbids self-closing syntax. It does not require
paired start/end tags for HTML void elements. The existing
[rule](../../../crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs)
and [fix](../../../crates/vize_patina/src/rules/opinionated/vue/html_self_closing/fix.rs)
already preserve that boundary. Both languages now describe the same policy;
all code examples and every other English byte are retained.

The existing reviewed-translation collector retains the entire authored
Japanese body during regeneration. Its existing test gains this article's
whole-example, API, link and bookmark contract. The More coverage manifest
changes only this route, and the foundation gains one companion link in its
existing Japanese editorial paragraph while retaining all 350 physical rows.

Exact-head Actions and the protected merge queue remain required. Fresh
official Docs publication and actual English/Japanese desktop/mobile reading
must qualify this article before deployed acceptance is recorded. Existing
Init and theme delivery keep their own source and publication evidence.
Whole-site Japanese/More review, installed CLI behavior and complete issue
acceptance remain unfinished.
