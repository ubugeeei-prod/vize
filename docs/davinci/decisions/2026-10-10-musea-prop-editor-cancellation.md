# Musea prop editor initialization cancellation

Issue: [#8453](https://github.com/ubugeeei-prod/vize/issues/8453).

## Decision

The Monaco mounted hook must recheck its container immediately after awaiting
the real module import. A removed control has already cleared that reference;
return before defining themes, registering providers or creating an editor.
Keep the existing initialization and disposal behavior for mounted controls.

## Regression

The existing Musea browser entry runs a genuine editor/gallery/preview test.
Its authored fixture supplies an ordinary label and object control defaults.
The browser delays the unchanged real Monaco response, removes the original
control through the editor UI, then releases that same response and creates a
new control. Module URL/full-byte hash, removed-before-resolution state, complete
current values, one live replacement editor and page errors are retained in the
existing always uploaded observations. The generated preview receives and
renders the remaining values through the actual message path.

The original hook deterministically reports `Cannot read properties of null
(reading 'parentNode')`. The corrected focused and complete local Chromium
entry passes with no errors. This completes the bounded source follow-up
recorded in [preview prop snapshots](./2026-10-10-musea-preview-prop-keys.md).

Fresh exact-head Actions, native Stack protected qualification, actual merge
and root-owned publication remain required. The explicit API/art fixtures do
not grant native compilation or installed publication credit. No manifest,
dependency, release authority or upstream changes.
