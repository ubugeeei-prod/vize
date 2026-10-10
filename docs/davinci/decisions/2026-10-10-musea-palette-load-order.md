# Musea palette response ownership

Tracking issue: [#8415](https://github.com/ubugeeei-prod/vize/issues/8415).

## Decision

Each props-editor load owns an increasing per-instance request version. Only
the latest version may publish its palette and restored/default values, report
an error, or finish loading. Comparing the component path alone is insufficient:
an earlier visit to A remains obsolete after A → B → A navigation.

This covers both development HTTP and static-gallery loading through the
existing `fetchPalette` API. Requests still settle normally; no transport,
pipeline stage, cancellation requirement, or package dependency is added.

## Regression and evidence

The real Chromium gallery imports the production `usePalette` composable. A
controlled fetch boundary holds eleven requests and resolves their genuine
Response bodies or rejects them in an authored order. The differential fixture
is `tests/_fixtures/differential/musea/palette-load-order.json`.

- An obsolete success preserves the selected palette, edited text, custom prop,
  and deleted prop.
- An obsolete rejection preserves a newer successful palette and error state.
- An obsolete completion leaves the latest request loading and unpublished.
- An obsolete success preserves the latest request's genuine API error.
- A → B → A retains the newest visit's edited value.

The browser contract persists complete fixture, request URLs and state snapshots
in `artifacts/musea-code/palette-load-order.json` before assertions. The unchanged
source/copy/scroll/theme contract runs afterward in the existing Musea Actions
entry. The original implementation failed with the previous component's palette
and defaults replacing the selected editor state. The corrected implementation
passed the full local Chromium contract; changed-file formatting/lint also passed.

Fresh exact-head Actions, protected merge-queue checks, actual merge and public
publication remain required. Local success does not establish release delivery.

## Remaining props-editor work

Deleting a generated prop and replacing it with a custom prop of the same name
still requires separate value/restoration ownership. Arbitrary custom prop names
also require safe copied-markup handling. This response-order fix claims neither
follow-up as complete.
