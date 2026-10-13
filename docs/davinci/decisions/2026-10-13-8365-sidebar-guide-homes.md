# Every guide has a named sidebar group

Issue: [#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

## Decision

No page is filed under the sidebar's trailing "More" section any more. Every
published page is listed in a named group in `docs/theme/i18n/sitemap.js`, or
is hidden because its parent page links to it in every locale that publishes
it.

The "More guides" group is split by what the reader is doing:

- **Format code** (new) holds the five formatter setting guides.
- **Rules by topic** (new) holds the eleven Vue and accessibility topic guides.
- **CLI & tools** (new) holds the CLI, Doctor, the Compiler Inspector, the MCP
  server, the WASM bindings, troubleshooting, and testing and feedback.
- **Project & internals** (the former "More guides" key, `architecture`) keeps
  stability, credits, contributing and the architecture pages.

The remaining pages join the group of the task they belong to. Project
initialization and editor setup go to Start. Shared Vue configuration,
scoped compiler settings and Experimentals go to Build an app. Lint execution,
type-checking performance and cross-file complexity go to Check code. Rule
options and the ESLint migration map go to Rule reference. The Musea guides,
source distribution and auto-imports go to Explore components.

The Experimentals reference and the Vue RFC detail page are hidden from the
sidebar. The Experimentals page links to both in English and Japanese.

## Reason

"More" had grown to forty unrelated pages in English and Japanese, and "More
guides" mixed command-line tools with project policy and crate architecture.
Eleven rule topic guides, the five formatter guides, shared Vue configuration
and scoped compiler settings had no inbound link from any listed page: the
"More" section was the only way to reach them, and nothing about its name
told a reader they were there.

## Scope and tradeoffs

Philosophy stays in Start and Content Mapper stays in Check code, as decided
under #8364. No page moved, was renamed or changed its content; only the
sidebar grouping, its labels and the group names changed.

French, Simplified Chinese and Portuguese publish fewer pages. They get the
same group names, and a group without pages in a locale is not rendered.

The Japanese Rule Options page inlines most option detail and links only the
type options page, so the four option pages are listed under Rule reference
rather than hidden behind it.

`tests/tooling/docs-navigation.test.ts` now fails when a page would fall into
"More" in any locale, or when a locale lists a page twice. The same test used
to spawn the theme unit tests with the parent runner's `NODE_TEST_CONTEXT`, so
the nested run exited 0 even when a theme test failed; it now clears that
variable and the theme tests gate the change.

The reviewed-page ledger in `docs/quality/more-guides-coverage.json` is
unchanged. It still records which of the former More pages were revised.
