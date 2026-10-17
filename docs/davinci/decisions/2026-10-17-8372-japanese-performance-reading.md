# Japanese performance reading path

Refs [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).
This is one bounded editorial delivery, not completion of the site-wide review.

## Inventory and ownership

The inventory at `ba44b935745efa05063da8cac0fc61782f89e0ac` contains 429 committed
Japanese Markdown pages: 255 single-file rule references, 66 project-rule
references and 108 other pages. These counts describe discovered files, not
completed editorial reviews. There are 26 blockquote packets, and 74 pages have
an explicit reviewed-translation marker. Only `architecture/performance.md`
still has a machine-translation marker. Absence of that marker does not establish
that a page has been reviewed.

The Japanese navigation input is `docs/theme/i18n/locales/ja.js`. Rule descriptions
and composed references are owned by `docs/scripts/rules/` and materialized by
`generate-patina-rules-page.ts`. Builds additionally generate the Japanese UI and
composable hubs from `npm/ui/scripts/generate-reference-docs.ts` and its
`reference-docs/` helpers. Those two routes are not in the committed-page count.
Benchmark sections and snapshots are owned by
`tools/benchmarks/scripts/publish-snapshot.mjs` and its locale/render inputs.
The complete Japanese performance article is also an input to the translation
regenerator: `collectReviewedTranslations` retains authored/reviewed documents.

## Change and preservation

Read the complete Japanese performance article beside its complete English
source. Replace broken sentences in the type-aware lint and type-checker
profiles, keep Croquis/Corsa/Glyph and package names untranslated, and clarify
memory allocation, work stealing and the separate cache/change-detection roles.
The development warning keeps its production-readiness limitation. Mark this
complete article reviewed so ordinary translation regeneration retains it.

All three complete generated benchmark blocks, both complete Rust examples,
89 inline code occurrences, 15 link destinations and all 36 numeric table rows
retain their original bytes or ordered values. Nine changed headings keep their
old native bookmarks. The existing desktop/mobile route campaign gains this
one Japanese article; no existing route or check is removed.

The inherited English and Japanese claim that all allocation budgets are zero
was stale. The current `[bench]` registry already records deterministic allocation
counts, including 9 for `armature_parse_small`; zero means a measured zero for
allocations, while zero wall time means the reference-runner baseline is pending.
Correct only this factual paragraph in both languages against the current
`budgets.toml` field documentation and registry.

The measurements remain historical measurements with their original caveats.
This editorial change does not rerun benchmarks, refresh implementation claims,
assert current performance, or establish diagnostic/runtime parity.

## Verification boundary

Two source laws exercise reviewed-translation retention and the real benchmark
section updater, and pin the original examples, contracts, measurements and
links. The updater's raw table padding differs from the committed formatted
blocks, so the regeneration law compares the complete authored context outside
its three explicit markers. The original complete marker blocks have their own
independent byte pin.

The installed pinned Ox Content 2.88.0 native Markdown renderer was used only to
identify the predecessor heading IDs and check alias preservation. This is not a
built-site browser or deployment result. Exact-source Actions, full native
rendering and desktop/mobile screenshot review, protected merge and deployed
reading-flow verification remain delivery requirements. The other Japanese
pages, remaining navigation/warnings and generated descriptions still require
their site-wide editorial and rendered review; #8372 remains open.
