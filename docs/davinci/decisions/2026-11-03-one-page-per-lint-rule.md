# One page per lint rule

Supersedes [Inline rule examples and native code annotations](./2026-10-08-inline-rule-examples-and-native-diff.md).

The maintainer found the rule reference hard to read: the sidebar listed dozens of
flat entries, and the catalogues sent readers through Bad/Good link columns into
pages of several thousand lines. The reference now has one page per rule, in every
locale, and the indexes are tables.

## Pages

- `rules/reference/<rule>.md` and `rules/project/<rule>.md` are the rule pages: a
  purpose sentence, a facts table (severity, presets, automatic fix, options), the
  Bad and Good examples with one-line reasons and changed lines annotated, the
  configuration, and related links. zh-CN, pt-BR and fr get the same pages from the
  existing translation tables; generation fails on any untranslated line.
- Category pages, `all.md` and `cross-file.md` are tables that link to rule pages.
  Category pages group rules by the smallest preset that turns them on. Each row
  keeps the old `#<rule-id>` fragment.
- The topic subsets (`vue-sfc`, `accessibility-core`, ...) and the "Rules by topic"
  sidebar group are removed: they repeated rules a category page already lists.

## Sources and drift

The English rule pages remain the examples `documented_rule_examples.rs` executes,
so readability costs no drift: every other locale's Bad/Good source must equal the
English bytes (docs tooling tests and the rendered-route review check this). The
fence info string carries `annotate="..."` display metadata; the Rust reader takes
the language from its first word.

Rule ownership is the namespace table `ruleSections` in `docs/theme/i18n/sitemap.js`.
The sidebar and the generator both read it, so a rule's page, category and sidebar
section agree. The staged content tree (`docs/.generated/content`) and the
`docs/content/generated/` catalogues are gone: Ox Content compiles `docs/content`
directly.
