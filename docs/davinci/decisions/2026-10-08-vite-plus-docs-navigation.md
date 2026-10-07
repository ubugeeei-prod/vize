# Vite+ adoption guides and goal-oriented navigation

Issue: [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

## Decision

The English and Japanese entry paths start with an existing Vue/Vite+ project.
The integration helper from `@vizejs/vite-plugin/vite-plus` owns native settings
in `vite.config.ts`. Ordinary Vite uses direct plugin options in that same file;
standalone CLI/LSP settings remain in `vize.config.ts` through `vize`.

Migration documentation shows source/target/effect/install mappings and literal
import, plugin, option, and package-script diffs. Generated tasks use `vp run`;
built-in `vp lint`, `vp fmt`, and `vp check` keep their Vite+ behavior. Existing
package scripts cause names such as `vize:lint`, which replacement scripts must
use to avoid calling themselves. No full diagnostic or plugin-option parity is
claimed.

Navigation groups describe user goals. Common paths remain mandatory in all five
locales; explicit English/Japanese start-path overrides add the authored Vite+
and migration pages without advertising missing translations. All additional
paths must exist in their declared locale. Start and the current group open;
other groups use native keyboard-operable disclosure. Generated component and
rule detail pages remain reachable through their indexes without adding every
leaf to the menu. Configuration details remain in linked standalone/compiler
references. Tables scroll within the content column instead of breaking paths
and option names or widening the page. Closed mobile navigation is hidden.
Entry pages hide their unused sidebar explicitly: the original offscreen sheet
appeared in full-page mobile captures and covered the six goal cards. The browser
gate requires those six authored cards to be visible and the entry sidebar hidden.
Home hero logos use a root-relative asset path in both authored locales; the
browser gate requires every same-origin image to decode, including the JA hero.

## Evidence and delivery

Registry exports/declarations for published `vize` and `@vizejs/vite-plugin`
0.435.0 were authenticated against their SHA512 tarball integrity. The actual
published helper passed config normalization and task/script collision checks;
this isolated config smoke did not execute compiler or diagnostic commands.
The [official run](https://viteplus.dev/guide/run),
[lint](https://viteplus.dev/guide/lint), and
[formatter](https://viteplus.dev/guide/fmt) guides establish built-in ownership.

`vp node docs/scripts/verify-navigation-render.mjs` serves real built site files,
checks English/Japanese desktop/mobile routes, local links and anchors, viewport
width, page errors, keyboard disclosure, and mobile menu interaction. It saves
actual Playwright PNGs and a JSON receipt. The existing Docs build runs it after
the production build and uploads the evidence. Local preview disabled only OG
generation to reuse cached dependencies; it grants no full Docs Actions credit.

The first Linux screenshot artifact exposed missing Japanese glyphs despite a
successful build. The Docs runner now installs CJK fonts before rendering, and
the theme declares native CJK fallbacks. The browser receipt records the actual
platform fonts used by visible authored Japanese text; Linux requires rendered
Noto CJK glyphs. Text presence and a successful screenshot alone are insufficient.

## Remaining work

- Attend exact-head Check and the full Docs workflow before queue admission.
- Verify the deployed site after the root-owned protected merge completes.
- Keep rule example/migration and component preview work source-owned in their
  separate changes; this slice does not satisfy every #6101 acceptance item.
- Extend authored entry guides to the other supported locales in a later slice.
- Compare checks on adopter projects before changing their CI commands.
