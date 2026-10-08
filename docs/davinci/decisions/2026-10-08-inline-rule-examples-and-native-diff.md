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

## Complete capture and native line layout

The full catalogue exceeds 500,000 CSS pixels. Chromium's single full-page PNG
capture times out for that page. Capture every vertical pixel using overlapping
viewport PNGs, retaining dimensions, offsets and complete PNG hashes. Short pages
keep their existing full-page capture. No route, theme, authored packet, screenshot
region, timeout or acceptance requirement is removed.

The capture helper may enlarge the viewport height to 8,192 pixels only after
proving the complete content tree retains the same node identities, source bytes,
element bounds and every element/text Range rectangle, plus document dimensions.
Keep the before/after comparison in Chromium and retain complete packet hashes.
On any change, restore the original viewport and prove its layout restored before
capturing at the original height. Always restore the original viewport and scroll.
A hosted control uses fixed element boxes and viewport-relative text: its boxes
stay equal while text rectangles change, requiring the original-height fallback.
This control runs inside the existing rendered-navigation verifier and browser.

Retain native annotation wrappers and newline nodes. Setting the native wrappers
to inline-block avoids an extra visual row between each preserved newline while
keeping the provider's add/remove colors and borders. Local Chromium checks of
14 real annotated blocks and 107 wrappers retain source bytes and exact adjacent
line-height spacing in desktop/mobile light/dark; whole deployed flows remain
required. Each catalogue rule also has its own complete inline code packet checked
against the existing authored reference, including both Bad/Good anchors.

## TypeScript ownership

All handwritten executable Docs `.mjs` modules migrate to erasable `.ts`, with
byte-identical move-only commits before type/import changes. The native Node
runtime executes those sources directly. Keep explicit strict compiler checks in
the existing Actions gate; renaming files alone does not establish type safety.
Preserve complete generated reference/catalogue bytes, original law bodies and
browser/provider invocation boundaries. The initial 51-module rule generator
migration passes strict compilation and 22 surrounding laws; all 680 generated
files and their path set retain exact SHA256 bytes. The integrated entrypoint,
materializer, render verifier and capture graph also pass native strict TypeScript
7.0.2 and 26 surrounding document laws. Actions invokes that already-pinned
compiler directly inside the existing Linux x64 Check job on every event; the
bounded Docs project excludes unrelated unfinished root-composite package graphs.
Existing JavaScript imports remain supported without claiming they were migrated.

All handwritten Docs `.mjs` sources are removed from the integrated tree.
The original 666-line translation entrypoint is divided into bounded typed
provider, concurrency, Markdown and translation modules. Native Node 22.18 and
24.14 retain exact normalization for all 1,015 authored content files and whole
ten-file scratch results for every provider; no actual HTTP request or authored
translation write occurs. Four offline regression laws retain complete fences,
YAML, links and provider behavior. Real Vite/Vue builds and Chromium interaction
capture for eight UI preview families retain all 258 asset/PNG/evidence bytes on
both Node versions. The typed render verifier also passes all four existing
EN/JA Getting Started desktop/mobile controls. After the signed #8293 bound-slot fix reaches main, regenerate the catalogues
from its retained reference source. This carries its new bound-slot Bad line and
correct implementation line 39 into both inline catalogues; no incoming example
is removed or rewritten. The earlier 680-file migration equality belongs to its
original input snapshot. Fresh source acceptance compares every complete current
reference packet. Complete hosted catalogue, queue and public acceptance remain
required. This applies the maintainer's repository-wide
`.mjs` removal direction without changing upstream or historical artifact sources.

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

## Source-growth correction

Actual parent3a7 Check37764777956/tooling2 rejects Check693 against base690
and the moved highlighter core535 as a new path. Preserve this failure. Keep
the native Docs compiler inside the existing mutually exclusive full/fast JS
steps, running once after installation on every event. Split unchanged language
handlers into a bounded helper loaded before the core; retain the same token
store, native wrappers and complete highlighted HTML. All 2,906 complete EN/JA
catalogue blocks produce byte-identical HTML before/after this extraction. The
11 actual syntax/native-annotation laws, 11 workflow laws and 10 source-growth
laws pass; the broader local navigation invocation stops on its old missing
typescript package link, so fresh installed Actions owns that acceptance. No
source-growth budget changes. Full native Docs types and scoped formatting/lint
pass; final-source Actions and actual deployment remain pending.
