# Japanese setup reading paths

This is a bounded continuation of [#8372](https://github.com/ubugeeei-prod/vize/issues/8372),
not a claim that the site-wide Japanese review is complete.

## Inventory and changes

Read the complete Japanese landing page, Getting Started, Configuration, Vite+
editor setup, VS Code integration, Static Analysis, and checker-performance guide,
plus the Japanese navigation labels and reviewed-translation generator.

- The landing page no longer describes every operation as implemented entirely in
  Rust. Canon delegates type diagnostics to Corsa project sessions. It now says
  Rust is the foundation, and distinguishes shared Vite tool configuration from
  the TypeScript project in `tsconfig.json`.
- Getting Started links to the existing reviewed Japanese editor/init guides,
  instead of labeling those steps English and sending readers out of Japanese.
- Configuration replaces the literal "共有の全体 ignore" and distinguishes
  syntax/type diagnostics from code navigation. Ordered exclusions and `!`
  negation retain their existing meaning.
- VS Code's literal "推奨される開始点" becomes an explicit lint-only trial step;
  its old bookmark remains an alias. The editor feature switch uses ordinary
  Japanese instead of "エディター支援バンドル". The startup navigation label now
  matches the page title.

The Vite+ editor, static-analysis and checker-performance guides already describe
their current settings and constraints clearly, so their prose stays unchanged.
The source contracts are Canon's Corsa session integration, Maestro's scoped lint
configuration/ignore tests, and the actual VS Code feature switches. This slice
changes no behavior, settings, code examples, other locales or support warnings.

## Verification and remaining work

All 13 complete code examples remain unchanged. The renamed VS Code heading keeps
its original target as an alias. Existing five-locale configuration, Japanese
editorial and navigation checks remain in place; the navigation test follows the
intended label change. Authored/reviewed documents are retained by
`collectReviewedTranslations`, and navigation labels are authored inputs. This
wording repair adds no parallel fixture of the existing pages. Desktop/mobile
reader checks, Actions, protected merge and actual public deployment complete this
slice's delivery.

The remainder of #8372 includes the full site inventory, generated descriptions,
remaining navigation/warnings and their rendered reading paths. This seven-page
audit does not qualify those other pages, and the issue stays open.
