# Existing Vite and TypeScript setup journey

Related issues: [#8370](https://github.com/ubugeeei-prod/vize/issues/8370) and
[#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

Keep the practical setup route centered on the existing `vite.config.*` and
`tsconfig.json`. Vite+ uses `compiler`, `typecheck`, `lint.vize`, and `fmt.vize`;
ordinary Vite and native consumers share top-level `vize` settings. Dedicated
formats remain in the detailed reference, with their existing same-directory
precedence, explicit `--config`, direct plugin/editor overrides, and opt-outs.

## Coverage

- All five locales receive the same Getting Started and practical configuration
  route, complete copyable recipes, native setting scope, and publication notice.
- French, Brazilian Portuguese, and Simplified Chinese previously kept the
  entire old reference on the practical route. Move those three files in a
  move-only commit, then split shared and compiler references below the unchanged
  source budget. Preserve all 42 original complete fenced examples.
- Native rendering retains all 172 original heading targets across the fifteen
  setup/configuration/plugin pages, including aliases for relocated sections.
- Update configuration ownership in CLI/workflow/plugin pages in all five
  locales; update existing English/Japanese migration and English editor/init
  guidance. Human review markers describe only reviewed sections where the
  remaining body still contains machine translation.
- Link tests use the actual maintained UI reference generator for generated
  targets and the native renderer for existing source targets and fragments.
  Missing localized routes never inherit an assumed English fallback.
- Localized Musea and library guidance uses the same whole native Vite examples
  as English. Fresh library init creates `vite.config.mjs`; existing Vite files
  are preserved and receive a manual snippet when needed. Dedicated JSON and
  code/Pkl compatibility remains explicit. Short formatter guides use `fmt.vize`.

Common Oxfmt settings are not advertised as native formatter options. Native
options use `fmt.vize` or top-level `vize.formatter`. TypeScript dependencies and
project semantics remain with the target package's TypeScript project.

## Delivery bounds

The shared native Vite discovery and config-free init changes are not in the
frozen emergency 0.440 source cut. Each locale explicitly says those features
are being prepared for the next release; published native tools still use the
existing dedicated format for custom shared settings. Remove that notice only
after an actual published source cut includes the product changes and its
installed CLI/editor configuration evidence passes. No version is guessed.

This genuine documentation child now follows the project-root layer #8445.
All five practical/reference routes distinguish Vite-owned root-relative
paths, dedicated config-directory paths, and invocation-relative CLI inputs.
Input-free commands use the selected Vite root, and ordered global ignores
exclude CLI discovery/editor lint while open editor files retain type and
navigation diagnostics. The original nine public CLI root/ignore laws passed
on the parent before this prose was adopted; full source/protected delivery
remains required. Automatic per-document nested Vite discovery is still
unfinished: retain package-root invocation, scoped `vize.entries`, and explicit
editor workspace folders until the later native layer actually qualifies.

Five focused regressions retain complete source examples, all original native
anchors, identical whole recipes in five locales, and actual rendered reading
targets. The Docs workflow captures all forty localized setup, configuration,
reference, plugin, workflow, Musea, and library routes plus three formatter
fallback routes at desktop/mobile widths using the existing
full-page capture and link checks, bound to the exact SSG manifest source. Keep
the inherited navigation and Open Graph gates too. Exact-head Actions,
protected Stack delivery, deployed all-locale
browser acceptance, and the included public release remain required. Historical
blog posts keep their original release context; optional dedicated format examples
stay in detailed references. #8370 is not closed by opening this PR.

The first complete browser run exposed a real French migration link returning 404. Untranslated guides now link explicitly to the existing English route,
with a localized language notice. The source regression fails on the old
implicit fallback and passes only when the actual destination exists. Restore
the lint rule-options reading section in all five practical guides, preserve
its old native bookmark, and retain complete typed options/bad-good examples
in the existing reference. Japanese uses a natural heading instead of the
old English heading literal. All 172 old targets must occur exactly once.
Every original code fence and all 43 desktop/mobile browser routes remain.

## Composition with delivered Japanese reading paths

The genuine config parent still follows the merged native JSX prerequisite.
Compose the now-delivered Japanese workflow prose from main with this child's
Vite recipe, retaining every historical bookmark. Preserve the original complete
18-line dedicated workflow recipe in both detailed references; the new custody
regression fails on the prior missing recipes and passes only with exact source
bytes. The original 172 heading targets and 42 whole configuration examples
remain unchanged. This adds a sixth regression without weakening the first five.

Keep the main Japanese and static-analysis browser gates alongside all 43
configuration reading routes, global navigation, and Open Graph verification.
The next genuine parent incorporates the actual merged Japanese browser
producer; no foreign implementation is duplicated in this documentation child.
Fresh exact-head source and full Docs Actions, protected Stack merge, release
availability, and deployed reader acceptance remain pending after this correction.
