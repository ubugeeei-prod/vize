# Inline rule examples and native code annotations

Tracking: [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

The maintainer requested one page containing the rule examples and Ox Content's
native highlighting for added and removed code. The EN/JA `/rules/all` catalogue
therefore contains all 251 single-file entries and 66 project entries, including
each purpose, configuration, support boundary, complete Bad/Good pair and authored
explanation. Shared project files and tracked graphs remain beside their examples.
The table jumps to unique rule, Bad and Good anchors within the same page.
Category navigation uses that catalogue. Existing reference URLs remain valid.

## Source ownership and compilation

The existing rule generators remain authoritative. They compose the same complete
reference text into `docs/content/generated/rules/en/all.md` and its Japanese counterpart.
Composition changes headings, anchors and relative prose links; fenced source bytes
are untouched. The deterministic generation check covers both full catalogues.
Whole-example laws compare every fenced block and complete Bad/Good explanation
against all 317 existing references in both languages, with unique target checks.

The pinned Ox Content 2.81.0 has no Markdown transclusion, frontmatter route alias,
or invoked custom Markdown-transformer hook. A tested native `@include` probe keeps
only the comment. Native route generation uses the input path relative to `srcDir`.
Do not invent an include directive or move the public route under `/generated/`.

The Vize docs configuration instead materializes `docs/.generated/content` from
the complete authored content tree, excluding only the two derivative catalogue
source files, and replaces exactly the EN/JA `rules/all.md`
inputs with the generated catalogues. Ox Content compiles that tree normally;
routes, native Markdown rendering, search and page metadata retain their authority.
The stage is disposable, ignored and rebuilt; authored files are never overwritten.
Missing catalogues fail rather than falling back to an example-free index.
The repository-facing index links to the public full catalogue. The existing
350-line authored-source ratchet and its generated-directory rule are unchanged.

Remove this staged-tree composition when the pinned provider supplies a qualified
native source-composition API that preserves the same routes and complete examples.
This docs build operation adds no toolchain pipeline stage or product serialization.

## Native diff annotations

Enable `codeAnnotations: true` with the current `highlight: false` option. Literal
diff prefixes become the pinned native attribute syntax, for example
`ts annotate="remove:1,2;add:3"`. Retain the complete original line content after
removing only diff notation. Copyable source has no prefixed `+` or `-` markers.
The installed provider recognizes `remove` and `add`; `diff-remove`, `diff-add`,
`-` and `+` silently produce plain code and are not valid replacements.

Client token highlighting preserves the provider's native line elements, attributes,
classes and newline nodes. Whole-block tokenization retains multiline language
context; balanced per-line token fragments populate the existing native wrappers.
The provider supplies line backgrounds/borders and unselectable line numbers.
The old engine moves in a move-only commit before its bounded DOM adapter changes.

## Acceptance and unfinished work

Require exact-head Actions, protected queue delivery and actual merge. Inspect the
built and deployed EN/JA catalogue and migration pages in light/dark mode, including
same-page navigation, complete shared projects, native diff rows, copied code and
repeat highlighting. Measure the full catalogue's rendering/highlighting workload;
a small detached probe does not qualify whole-page responsiveness.

Local qualification passes 16 document/custody laws and 10 native/client laws.
Both complete native catalogues have 317 pairs and 1,453 code blocks without renderer
errors. Chromium preserves all source bytes after highlighting; actual edited pages
retain 14 native diff blocks and 107 native line wrappers in light/dark desktop/mobile
checks. These local results do not establish hosted or public acceptance. Exact-head
Actions, protected actual merge and deployed browser verification remain pending.
Page-specific OG image work is tracked independently under #6101.
