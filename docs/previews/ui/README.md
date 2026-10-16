# Component documentation previews

The component hub is generated at `/guide/ui` in English and each locale. Each
family page uses the basic SFC maintained beside its component source. Only
module specifiers change: catalog entry imports become public `@vizejs/ui/*`
imports. The entire example is used both as copyable documentation and as the
interactive iframe source.

`build-config.ts` binds those public imports to the current workspace catalog
entry files. It uses the same Vue compiler plugin as the UI package build, not
HTML imitations or a previously published component version. The optional CSS
comes from the matching workspace styles. It does not change the documentation
site's renderer or Vue compiler configuration.

The docs build generates the browser app under
`docs/public/component-previews/app/`, which the static site copies unchanged.
All basic examples receive live previews. Eight common examples additionally
receive actual Playwright screenshots: Button, Input, Checkbox, Switch, Tabs,
Dialog, Alert, and Card. These are illustrative captures after the interactions
in `capture.ts`, not pixel-difference baselines.

```bash
vp run --filter './docs' generate:ui-previews
vp node --test tests/tooling/lib-reference-docs.test.ts
vp run --filter './docs' check:ui-docs
```

The preview build fails on browser errors or warnings, broken interactions,
missing public entry mappings, or horizontal overflow at a 360px viewport.
It also verifies dialog Escape/focus return and the theme selector. Generated
browser evidence records the source and successful scenarios. It does not claim
that every component has been audited for accessibility or every advanced
variant has been exercised.

After SSG completes, `check-site.ts` checks generated English/Japanese
documentation routes, their local links and images, and an embedded Button's
state update at desktop/mobile widths. The production docs build runs both
browser checks with its normal OG-image configuration.

## Composable examples

One hundred twenty-five complete SFCs in `npm/compose/core/examples/` demonstrate disclosure,
bounded quantities, debounced local search, blur validation, undo/redo, and
pagination, plus async results, interval/timeout controls, browser storage,
media queries, document focus, form submission, multistep navigation, bounded
selection, cyclic tips, individual/group focus, outside dismissal, scoped keyboard
shortcuts, retargetable native input observation, explicit clipboard operations,
element scrolling, panel size observation, bounded-root card visibility, native
border-box geometry, delayed pointer guidance, held actions, typing focus, actual
connectivity, color preference, viewport-dependent detail and card layout,
actual pointer snapshots, element-relative mouse geometry, bounded note dragging,
and native document selection in an editable excerpt, plus a throttled release-title
preview with explicit pending, native flush/cancel and retained draft states, and
locale-aware currency totals, speaker lists and unread-message counts, a guarded
checkout state machine, a price change compared with its previous value, a sign-in code
resend cooldown, and relative meetup dates, plus collections (arrays, sets, maps, queues,
stacks, sorting and memoization), date, language and text formatting, clocks, confirmation
dialogs, async step queues, form field wiring, URL query/hash/search state, cookies and IndexedDB, and page resources (CSS variables, title,
favicon, style and script tags, object URLs, images), network (fetch, event streams, WebSockets,
broadcast channels), workers, compression, animation frames, idle and prioritized scheduling, Web
Locks, view transitions, performance entries, storage estimates, the visual viewport, user
activation, the Navigation API, drag-and-drop, file pickers and media controls. Device and permission examples (Bluetooth,
HID, serial, USB, MIDI, gamepads, payments, passkeys, contacts, eye dropper, barcode detection,
file system access, fullscreen, notifications, permissions, sharing, vibration, wake lock, screen
details, screen and camera capture, speech, service workers, push and built-in AI) are checked
against the browser's own feature detection, because the preview browser has no devices.
Their displayed source and compiled preview share the same complete SFC packet,
with a SHA-256 identity exposed by each iframe. They use public composable
imports and can be copied into a Vue 3.5+ project. The preview shell's CSS is
optional presentation; the examples themselves use native controls.

The virtual-list catalogue uses real native scrolling over 1,000 local records,
with a bounded rendered slice, search/empty states and keyboard jump controls.
Its checks verify actual row dimensions, native Home/End/PageDown, ARIA positions
and total counts, and retained input/list focus after filtering and scrolling.

The async-resource delivery quote demonstrates native abortable local producers,
retained successful data, failures, cancellation, supersession and reset. The
capture checks real timer cancellation and execution results, then unmounts the
actual hydrated Vue app after its screenshots and checks for late browser errors.

`build-ui-previews.ts` typechecks those SFCs against workspace sources before
compilation. `prerender-composables.ts` uses the existing Vue/Vite configuration
and actual Vue server renderer. A fresh plugin instance builds the browser app.
The client hydrates the matching markup; the checks require all initial
elements and complete markup to survive hydration, without warnings or errors.

`capture-composables.ts` runs every example at 720px and 360px in both initial
and interacted states. It checks keyboard activation/blur, state boundaries,
debounce timing/flush/cancel, validation, undo/redo/batching, and pagination.
The effect groups in `interactions-async-effects.ts` and
`interactions-browser-effects.ts` also check producer rejection/retry and stale
completion, start/pause/reset, replace/cancel/single delivery, actual storage
across reload/reset, actual matchMedia at resized viewports and changed queries,
and actual document focus through Tab/programmatic focus/blur.
Browser clock control advances the actual timers deterministically; browser
hosts are not replaced with mocks. `interactions-form-state.ts` checks validation
and reset, step boundaries and preserved input, stable selected keys after data
replacement, and cyclic list boundaries. `interactions-dom.ts` additionally checks
actual descendant focus, frozen stopped observations, ignored/inside/outside clicks,
drag origin, Escape focus return, key-repeat suppression, target replacement, and
listener pause/resume. `interactions-observed-browser.ts` checks real clipboard
contents and denied/restored browser permissions, replacement copy feedback,
native keyboard scrolling and element boundaries, actual content-box dimensions,
observer target replacement, and pause/resume/stop of native visibility observation.
`interactions-scoped-input.ts` compares actual border-box geometry, explicitly
refreshes transformed positions, and checks reset/remount and stopped snapshots.
It exercises actual delayed pointer events and retargeting while the advice stays
available through native Enter and Space. Held actions deliver before release,
cancel on early release or threshold movement, and stop without removing ordinary
native activation. The typing shortcut respects Space, navigation, modifiers,
and native field editing while the local guide filter remains usable after stop.
`interactions-preference-browser.ts` switches actual browser connectivity while
preserving a native draft and local review action, verifies real event timestamps,
and changes actual system color media, the document attribute, computed sample
colors, and the dedicated storage key across reload. A chosen mode is not labelled
saved when persistence is unavailable. `interactions-environment.ts` crosses actual
480/768px breakpoint and 640px grid boundaries, compares native window dimensions,
and preserves explicit layout choices, measurement visibility, and exact drafts.
`interactions-pointer-selection.ts` checks actual PointerEvent coordinates and
pressure, target removal/replacement, transformed relative rectangles, outside
freezing and independent client subscriptions after stop. It exercises a native
drag handle with whole-note bounds, reactive axis/disabled state, explicit
keyboard placement, listener removal and exact draft retention. Native keyboard
selection and editing refresh the same platform Selection object; real Range
geometry, no ranges and a collapsed caret remain distinct.
One hundred forty-eight real PNGs and a JSON receipt record source hashes, image hashes,
and individual interaction results under the ignored
`public/component-previews/composables/` build output. Each reference page
offers initial/after images in a native disclosure alongside its live preview.

`capture-unavailable-storage.ts` additionally launches actual Chromium with
localStorage disabled, without replacing the host. Its two mobile captures and
separate receipt prove that keyboard selection still changes in-tab state and
reload returns to the default. The storage page shows that fallback capture.
Field status reports only recorded validation until the next blur. The batching
law uses meaningful whitespace trimming and requires Undo to restore it exactly.

After the real SSG build, `check-site.ts` verifies forty-seven routes at desktop
and mobile widths, including the thirty-seven exact displayed SFC packets and their
iframe identities, then runs the unchanged composable interaction checks.
The clipboard route also exercises keyboard copy/read inside the actual rendered
iframe, including denied and restored browser read permission.
Its default receipts and captures live in repository-level
`docs-render-evidence/component-previews/`, regardless of whether the command
starts from the workspace root or the docs package. The existing Actions
render-evidence artifact retains this second SSG acceptance pass.
The existing Docs Actions build executes both paths. To check deployed content:

```bash
vp node docs/previews/ui/check-site.ts --site https://vizejs.dev --output docs-render-evidence/deployed-composables
```

The supplied deployed URL must expose the expected source hashes; an older
deployment fails instead of receiving current-source credit. These thirty-seven
examples are a bounded acceptance slice, not complete live coverage of the
composable catalogue. #8374 and the broader #6101 remain open for that work.
