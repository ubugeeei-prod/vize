# Japanese investigation and feedback guides

Refs [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).
This is a two-page editorial continuation, not completion of the site-wide review.

## Reviewed coverage

Read both complete Japanese guides and their English source documents at main
`b0867d5e60a39244859b5089dbcf420d4cb84a99`:

- `guide/compiler-inspector.md`: explain the order of input, target selection,
  whole-output comparison and sharing. Replace untranslated payload/head jargon
  with the data or branch the reader actually chooses. Keep all original complete
  fenced examples and the three original heading targets; renamed headings retain
  explicit aliases. Clarify the existing Vue 3-only development comparison limit.
  Align its authored Japanese sidebar label with the page title, and name GitHub's
  `base` and `compare` selectors explicitly.
- `guide/testing.md`: connect compiler inspection, type/lint/editor reports,
  performance measurements and project proposals. Add a small counter SFC as a
  starting point, a copyable report outline, and the existing profiling command.
  The counter is an illustrative reduction, not an asserted failing fixture.

The CLI options and comparison limit are owned by
`crates/vize/src/commands/inspector.rs` and `inspector/compare_error.rs`.
The inspector's actual tabs, payload and sharing controls are in
`playground/src/features/inspector/InspectorPlayground.vue` and `share.ts`.
The existing counter SFC preset in `playground/src/presets.ts` supplies the
`ref(0)` state/event example. The profiling command is already documented in
`guide/check-performance.md` and implemented by the checker command.

No configuration migration page, compiler behavior, snapshot, support tier or
performance result is changed. Other locales retain their complete sources.
Both reviewed markers remain generation inputs: `collectReviewedTranslations`
retains their entire bodies before rebuilding a locale directory.

## Delivery and remaining scope

Keep the existing frozen compiler-guide example/heading checks unchanged.
The current Docs build calls `docs/previews/ui/check-site.ts`, not the older
navigation-render manifest. Add both Japanese guides to that existing
desktop/mobile loop, keeping all 11 previous routes and composable examples.
Its real-page heading, body-overflow and local-link checks cover the edited
reading paths. Four full-page screenshots join its existing uploaded evidence;
wait for the Japanese fonts before capturing them. No workflow or job is added.

The first exact-source Docs build passed, but its captured pages still showed
the article's finite fade-in animation. Preserve those initial images separately;
use the same screenshot `animations: "disabled"` option as `capture-page-render.ts`
to finish finite animations when capturing the successor. Fresh source checks,
Docs and four actual successor images are required; initial green CI does not
qualify the changed source.

Exact-source Actions, rendered-page review, protected merge and deployed-page
verification remain delivery evidence to collect; this source record does not
claim those checks have passed. The remainder of #8372 includes the full site,
remaining warnings, navigation labels and generated descriptions. Keep it open.
