# UI family closing conditions

Issue: [#3134](https://github.com/ubugeeei-prod/vize/issues/3134). Family issues:
[#4900](https://github.com/ubugeeei-prod/vize/issues/4900) through
[#4913](https://github.com/ubugeeei-prod/vize/issues/4913).

## Decision

A `@vizejs/ui` family issue closes when every line of its inventory is implemented
and every component passes the gates that the family catalog enforces
(`qualityGates` in `npm/ui/src/catalog`): behavior contract, mounted DOM, type
inference, SSR, hydration, DOM / SSR / Vapor compile, tree-shaking, and bundle size.

Two requirements of the previous per-family definition of done move into
cross-cutting issues that cover every family at once:

- [#9006](https://github.com/ubugeeei-prod/vize/issues/9006): Musea stories, browser
  tests, and visual regression for every family and preset.
- [#9007](https://github.com/ubugeeei-prod/vize/issues/9007): native Vapor SSR,
  removing the Vize SFC Vapor+SSR fallback, for every family.

The remaining items of the previous definition of done stay listed in #3134 as
quality targets that each component PR is reviewed against. They are not closing
conditions for a family issue.

## Reason

Under the previous definition no family issue could close: in October 2026 no
`@vizejs/ui` family had Musea stories, the story testbed listed every browser and
VRT artifact as planned, and native Vapor SSR was still in progress
([#8962](https://github.com/ubugeeei-prod/vize/pull/8962)). Both requirements are
built once, as shared infrastructure, rather than family by family. Tying every
family issue to them kept finished inventories open and hid which work actually
remained.

## Tradeoff

A closed family issue no longer proves Musea, VRT, or native Vapor SSR coverage
for that family. That status lives in #9006, #9007, and the story testbed
(`vp exec node scripts/story-testbed.ts check`), which reports each family's
story, browser, and VRT artifacts.

## Decided by

The maintainer, on 2026-10-14, choosing between keeping the previous definition and
this split.
