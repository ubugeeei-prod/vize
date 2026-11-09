# Style languages on the levels: audit and target (#11550, #11551, #11552, #11553)

Audited from `086b0517810` (origin/main, 2026-11-09). This record covers the four
style-language gates together, because most of the work is one pipeline with a
dialect module per language: `inputs.sass` #11550, `inputs.scss` #11551,
`inputs.less` #11552 and `inputs.stylus` #11553. Each issue body stays the
acceptance criterion. This record decides what "natively on the levels" means for
them and the order of work; it closes none of them.

## Today

Every product finds style blocks through the legacy `vize_atelier_sfc::SfcDescriptor`
(block splitting itself already runs in `vize_l1::container`). After that, each
product spells its own language list and its own scanner.

| Product               | CSS (baseline)                                                                                                        | SCSS and Less                                                                                                                                              | Sass (indented) and Stylus                                                                                                          |
| --------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Compiler, CLI         | Scoped, module and `v-bind()` output from `vize_atelier_sfc::style` and `::css` (legacy).                             | `vize build` re-parses each SFC with legacy `parse_sfc`, runs the project's `sass`/`sass-embedded` or `less` in one Node process, then the CSS path above. | Same, with `sass` (indented syntax) or `stylus`; `styl` is an alias.                                                                |
| Compiler, bundlers    | Vite, Rollup, webpack, Rspack hand the block to the bundler's CSS pipeline; scoping and `v-bind()` run through NAPI.  | The bundler preprocesses; a post-transform scopes the result with `vize_atelier_sfc::vite_plugin` (legacy).                                                | Same.                                                                                                                               |
| Compiler, native path | `vize_atelier_sfc::native` accepts plain blocks and a few scoped simple-class families on the L1 CSS parser.          | Refused: `StyleCompilationUnavailable`; `vize_l1::css::StyleSyntax` returns `UnsupportedLanguage`.                                                         | Refused, as for SCSS.                                                                                                               |
| Formatter             | Glyph prints the block with lightningcss.                                                                             | Kept, outer indentation normalized. Opt-in requote and reindent use Glyph-local scanners (`style/preprocessor_layout.rs`, `style/quotes.rs`).              | Kept verbatim apart from outer indentation, as Prettier does.                                                                       |
| Linter                | `css/*` rules parse the block with lightningcss inside Patina.                                                        | `css/*` rules skip the block. SFC rules (`vue/no-preprocessor-lang`, `vue/require-scoped-style`) read `lang`; the first re-parses with legacy `parse_sfc`. | Same. Directive comments accept `//` for `stylus` but not for its `styl` alias.                                                     |
| Type checker          | `$style` gets the authored classes from `vize_canon::css_classes`; `__VLS_StyleScopedClasses` uses the same scanner.  | `$style` falls back to `Record<string, string>`; scoped classes run the plain-CSS scanner over SCSS, so nested names such as `&__title` are missed.        | Same fallback; scoped classes miss every indented rule.                                                                             |
| LSP                   | Maestro-local lexers for completion and hover contexts, folding and document colors; `$style.` completion uses Canon. | Braced lexer; `$style.` completion uses Canon's nested SCSS and Less scanners.                                                                             | Indented lexer; `$style.` completion uses Canon's indented scanners. Folding is brace-only and treats `styl` without line comments. |

None of the four languages is a dialect in `docs/davinci/plan/deletion-readiness-policy.json`
or in `tests/differential/source-dialect-coverage.ts`, so no product corpus can show
rows for them yet. Incidental SCSS blocks exist in linter and LSP fixtures; there are
no Sass rows anywhere.

## Target

Preprocessors stay upstream: `sass`/`sass-embedded`, `less` and `stylus` keep running
through the existing Node integration in the CLI and through each bundler's CSS
pipeline. Davinci never reimplements them. Everything Vue-specific around them runs
on the levels:

- **L1 owns the style language.** `vize_l1::css::lang` resolves one `StyleLanguage`
  from the block's `lang` (absent, `css`, `postcss`; `scss`; `sass`; `less`; `stylus`
  or `styl`; anything else stays opaque). Each language has a dialect module under
  `css/lang/` that supplies its comment, interpolation, block-structure (braces,
  indentation, or both for Stylus), mixin/control-flow and parent-selector rules.
  The core module stays language-neutral and produces the structure products read:
  rule preludes, declarations, at-rules and comments with file-absolute spans, and
  the facts derived from them: authored local and global class names with `&`
  resolved, and `v-bind()` call spans.
- **L2 owns the Vue facts.** `vue.css-bind` ops come from the L1 `v-bind()` spans
  (today `vize_l1_to_l2::lower::css` rescans the text), and CSS-module export facts
  per module name come from the L1 class facts.
- **L4 owns the emitted CSS.** Scoped selector rewriting, CSS-module class names and
  export maps, `v-bind()` variable rewriting and keyframe scoping run on the plain CSS
  a preprocessor returns, the same writer for every language. This is the compiler
  half of `levels.l4` (#6840) and replaces `vize_atelier_sfc::style`, `::css` and the
  `vite_plugin` scoping helpers.

What each product needs for "no legacy-backed path":

| Product      | Native requirement                                                                                                                                                                  |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Compiler     | CLI preprocessing collects blocks from L1 container style views, not `parse_sfc`; scoped, module and `v-bind()` output for preprocessed CSS comes from the L4 writer, CLI and NAPI. |
| Formatter    | Style blocks come from the L1 container (#6847); SCSS and Less requote and reindent read L1 comment and string spans; Sass and Stylus keep the verbatim policy.                     |
| Linter       | `css/*` rules read the L1 structure, so they run on every language with spans on authored bytes; SFC rules read `StyleLanguage`.                                                    |
| Type checker | `$style` and scoped-class types come from L1 class facts per language, so preprocessor modules are typed instead of falling back.                                                   |
| LSP          | Completion and hover contexts, folding, document colors and `$style.` completion read the L1 structure and facts; Maestro keeps no style lexer of its own.                          |

Differential rows: the harness lane (#6891, #6892) owns dialect registration. The
style dialect ids are `scss-style`, `sass-style`, `less-style` and `stylus-style`,
detected from `<style lang>` like `pug-template` is from `<template lang>`. This lane
adds the per-product rows and the zero-fallback evidence (#6854) for them.

## Order

1. `vize_l1::css::lang`: `StyleLanguage` plus the class facts for all five languages,
   moved from `vize_canon::css_classes`; Canon and Maestro read them, and the Canon
   module is deleted.
2. Typed `$style` and scoped classes for preprocessor blocks from those facts.
3. `v-bind()` spans as an L1 fact; L2 lowering reads it; product language lists
   (Patina directive comments, Maestro folding and colors) use `StyleLanguage`.
4. The L1 structure (preludes, declarations, comments) per dialect, then Patina
   `css/*` rules, Maestro contexts and Glyph's preprocessor layout move onto it.
5. CLI preprocessing on L1 container views; the L4 style writer with the l2l4 lane,
   then the bundler post-transform and the native compile path switch to it.
6. Style dialect ids and rows in each product corpus, then zero-fallback evidence.

The gates pass only when all five products satisfy each issue body and a release
contains the work.
