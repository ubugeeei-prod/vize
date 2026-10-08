# Page-specific documentation Open Graph images

Decision paired with [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

Non-home documentation routes need their own title, useful description, section,
language and image identity. Keep the homepage's existing paper/ink palette,
logo composition and typography. Ordinary pages use smaller, wrapping titles
for long rule names, localized section/language labels and their visible route.

## Authenticated failure and provider boundary

This work starts at source `8106d61dfd243638a4d9940be0089276bd195940`.
Public English/Japanese Getting Started and component-casing rule pages omit both `og:image` and
`twitter:image`. The successful [main Docs build](https://github.com/ubugeeei-prod/vize/actions/runs/37739939310)
logs a failed OG batch: Ox Content 2.81.0's bare Rolldown Vue loader parses
Vize's virtual CSS module as JavaScript. Its SSG path catches the error and
removes image references while allowing publication.

The pinned, published 2.81.0 API accepts explicit page props through
`generateOgImages` but has no SSG page-metadata callback. SSG reads Markdown
directly; a preceding Vite transform does not enrich this path. Its automatic
props do not contain route or locale, and descriptions only use frontmatter.
Its browser launch also ignores `PUPPETEER_EXECUTABLE_PATH`. Do not pretend that
declared but unimplemented transformer hooks supply these capabilities.

## Explicit compatibility adapter

Run a Vize-owned step after SSG. Compile the single `theme/og.vue` authority with
the real project-root Vite/Vize SSR pipeline. Native SSR intentionally omits
client style imports; emit the same Vue authority's matching scoped stylesheet
through its real native client/library CSS build.
Pass that compiled component and its emitted stylesheet through a generated
TypeScript SSR wrapper to the published `generateOgImages` API. No official Vue
compiler fallback or alternate copy of the homepage design is introduced.
Disable automatic provider OG generation to avoid duplicate or silently failed
work. This wrapper is a documented 2.81.0 integration adapter, not a claim that
the provider's bare Rolldown path works.

Read metadata from the actual generated HTML with Chromium's DOM parser.
Retain authored descriptions; pages without one use their first meaningful
content paragraph, with a localized title/toolchain fallback only when needed.
Derive category and locale from the route; check the rendered HTML language.
Include route, locale, text and the compiled template/CSS/logo fingerprint in
image identity. Disable provider render-cache reuse so browser/font changes
cannot silently reuse old image bytes. Source Markdown and rule generators
remain their existing authorities.

This is a native Stack child of the inline rule-catalogue/native-rendering change.
Use its shared `CATALOGUE_SOURCES` authority to exclude only the two derivative
Markdown inputs composed into the existing English/Japanese `/rules/all/` routes.
Every other source page remains in the independent whole-site inventory, and
metadata comes from the actual native-rendered output, including both catalogues.

Check/install the provider-resolved Playwright Chromium rather than treating a
different system browser as proof that its image API can launch. Reject duplicate
routes, missing titles, missing/error results, and invalid or non-1200×630 PNGs.
Only then write escaped absolute OG/Twitter image URLs, matching descriptions,
image dimensions/alt text, page URL, site name, type and locale into each page.
Retain a route/metadata/image manifest and representative generated PNG evidence.

## Acceptance and remaining work

Unit controls cover locale/route identity, authored/content descriptions,
escaping, metadata replacement and PNG validation. Hosted Docs acceptance must
validate actual generated metadata and decode every image, comparing the complete
route inventory against source Markdown. Retain English, Japanese, long-rule and
homepage examples for visual review; their PNG bytes must differ. Authored CLI
and Chinese homepage titles are checked independently where visible headings
differ. Bypass task-result caching for this source-SHA-bearing build. Publication is
complete only after an eligible current-main build actually deploys and public
HTML plus image responses pass the same metadata/dimension checks.

The adapter can be removed when a pinned provider supports project-root native
Vue SSR/CSS, explicit route/locale metadata and fatal image acceptance with
equivalent representative and whole-site evidence. Upstream remains read-only;
changing a version alone does not establish that acceptance.

Implementation, hosted image review, protected delivery and public deployment
are unfinished until their actual receipts are recorded on the paired issue.
