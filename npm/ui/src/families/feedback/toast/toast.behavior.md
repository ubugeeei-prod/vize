# Toast behavior contract

Normative state x input -> outcome table for `@vizejs/ui/toast`: the pure
`createToastStore` queue, `useToast`, `toast-provider.vue`, `toast-viewport.vue`
(`Toaster`), `toast-root.vue`, `toast-title.vue`, `toast-description.vue`,
`toast-action.vue`, and `toast-close.vue`. Every row names the test that proves it.

| Area             | State x input                                                      | Observable outcome                                                                                                          | Proven by                                                                                                             |
| ---------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| SSR-safe store   | store created, never started, toasts added                         | ids are deterministic (`toast-1`, ...); no timer runs, so server rendering never schedules work                             | `toast-store.test.ts` › `store is inert until started and generates deterministic ids`                                |
| Auto dismiss     | started store, visible toast, duration elapses                     | toast closes with reason `timeout`; `onAutoClose` then `onDismiss` fire once; `remove` drops it                             | `toast-store.test.ts` › `started timers auto-dismiss visible toasts ...`                                              |
| Limit and queue  | more open toasts than `limit`                                      | extra toasts wait in `queuedToasts` ordered high > normal > low, then creation order; a closed slot promotes the next       | `toast-store.test.ts` › `limit queues extra toasts and high priority jumps ...`                                       |
| Queued timers    | toast waiting in the queue                                         | its timer does not run until shown; dismissing a queued toast removes it immediately                                        | `toast-store.test.ts` › `queued toasts keep their full duration ...`                                                  |
| Pause/resume     | overlapping pause reasons, then resume                             | timers stop while any reason is active and resume with the remaining time; `stop` freezes every timer                       | `toast-store.test.ts` › `pause reasons overlap and resume keeps the remaining time`                                   |
| Replace/update   | `toast({ id })` for an existing id, or `update(id, patch)`         | toast is replaced in place, `revision` increments, timer restarts; loading defaults to `Infinity`; unknown id returns false | `toast-store.test.ts` › `an explicit id replaces the existing toast ...`                                              |
| Promise toast    | `promise(p, { loading, success, error })`                          | loading toast becomes success (value-derived) or error (reason-derived) in place; the original promise is returned          | `toast-store.test.ts` › `promise toasts move from loading ...`; `toast.test.ts` › `promise toasts render loading ...` |
| Dismiss all      | `dismiss()` without id                                             | every open toast closes once; a second call reports `false`; `clear` empties the store                                      | `toast-store.test.ts` › `dismiss without an id closes every toast ...`                                                |
| Validation       | invalid limit, duration, or blank action `altText`                 | throws `VIZE_UI_TOAST_OPTION`                                                                                               | `toast-store.test.ts` › `configure and options validate their inputs`                                                 |
| Hotkey helpers   | hotkey tokens and keyboard events                                  | labels read `Alt+T`; modifiers match event flags and other tokens match `code` or `key`                                     | `toast-store.test.ts` › `hotkey helpers format labels ...`                                                            |
| Region           | ToastViewport renders visible toasts                               | `<section aria-label="Notifications (F8)" tabindex="-1">` wraps an `<ol>` of `<li role="status" aria-live="off">` items     | `toast.test.ts` › `renders a labelled region with list items and every default part`                                  |
| Exit             | toast closes without CSS exit motion                               | presence completes immediately and the store removes the toast; with motion it waits for `animationend`/`transitionend`     | `toast.test.ts` › `visible toasts auto-dismiss after their duration ...`                                              |
| Pause sources    | pointer over region, focus within region, `visibilityState` hidden | every timer pauses (`data-paused="true"`) and resumes with remaining time afterwards                                        | `toast.test.ts` › `hovering or focusing the region and a hidden page pause timers`                                    |
| Hotkey           | document keydown matching `hotkey` (default F8)                    | focus moves to the region and the event is prevented; the region name includes the formatted hotkey                         | `toast.test.ts` › `the hotkey moves focus to the region ...`                                                          |
| Keyboard dismiss | Escape on a focused dismissible toast, or close button click       | toast dismisses and focus returns to the region; non-dismissible toasts ignore Escape and render no default close button    | `toast.test.ts` › `Escape and the close button dismiss ...`                                                           |
| Swipe            | pointer drag along `swipeDirection`                                | `data-swipe` moves start -> move -> end/cancel, `--vize-toast-swipe-move-*` publishes offset; past threshold dismisses      | `toast.test.ts` › `swiping past the threshold ...`; `vertical swipe directions publish the y offset`                  |
| Action           | action button activation                                           | `action.onClick` runs; unless `preventDefault()` was called the toast dismisses with reason `action`                        | `toast.test.ts` › `action activation runs the toast callback ...`                                                     |
| Stacking         | `limit` smaller than open toasts                                   | only `limit` items render; dismissal promotes the highest-priority queued toast                                             | `toast.test.ts` › `limit stacks visible toasts and promotes queued ones by priority`                                  |
| Announcements    | new or updated toasts                                              | embedded live region speaks `title. description. altText`, polite by default and assertive for `error` or `high` priority   | `toast.test.ts` › `new toasts are announced politely ...`                                                             |
| Custom render    | viewport default slot                                              | slot receives typed `toast`, `index`, `count`, and `dismiss`; parts fall back to toast text                                 | `toast.test.ts` › `custom slot rendering receives typed data ...`; `ToastAction requires alt text ...`                |
| Provider guard   | useToast or parts outside ToastProvider/ToastRoot                  | throws `VIZE_UI_CONTEXT_MISSING`                                                                                            | `toast.test.ts` › `useToast and parts require a provider`                                                             |
| SSR              | isolated server requests with toasts created during setup          | byte-identical markup; hydration reuses every item with no warnings; announcements happen only after mount                  | `toast-ssr.test.ts`                                                                                                   |
| Types            | typed store, promise phases, options                               | `Data` flows to records and slots; promise `success` receives the resolved type; closed unions reject unknown values        | `toast.types.test-d.ts`                                                                                               |
| DOM/SSR/Vapor    | authored SFCs compile                                              | every SFC compiles in the dom, ssr, and vapor renderer lanes                                                                | `scripts/check-renderers.ts`                                                                                          |

## Public Provider Props

| Prop                | Type                                  | Default           | Contract                                                            |
| ------------------- | ------------------------------------- | ----------------- | ------------------------------------------------------------------- |
| `store`             | `ToastStore<Data>`                    | `undefined`       | External store; otherwise a request-local store is created.         |
| `duration`          | `number`                              | `undefined`       | Default duration in ms; `undefined` keeps the store default (5000). |
| `limit`             | `number`                              | `undefined`       | Visible limit; `undefined` keeps the store default (3).             |
| `label`             | `string`                              | `"Notifications"` | Region name prefix.                                                 |
| `hotkey`            | `readonly string[]`                   | `["F8"]`          | Keys that focus the region.                                         |
| `swipeDirection`    | `"down" \| "left" \| "right" \| "up"` | `"right"`         | Dismiss swipe direction.                                            |
| `swipeThreshold`    | `number`                              | `50`              | Pixels a swipe must travel to dismiss.                              |
| `pauseOnPageHidden` | `boolean`                             | `true`            | Pause timers while the document is hidden.                          |

## Public Viewport Props

| Prop           | Type      | Default | Contract                                         |
| -------------- | --------- | ------- | ------------------------------------------------ |
| `pauseOnHover` | `boolean` | `true`  | Pause timers while a pointer is over the region. |
| `pauseOnFocus` | `boolean` | `true`  | Pause timers while focus is within the region.   |
| `announce`     | `boolean` | `true`  | Speak new and updated toasts via live region.    |

## Parts And Data

| Target     | Public contract                                                                                                                                 |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Provider   | `data-vize-ui="toast-provider"`, `part="provider"`                                                                                              |
| Viewport   | `data-vize-ui="toast-viewport"`, `part="viewport"`, `data-paused`; list `data-vize-ui="toast-list"`; announcer `data-vize-ui="toast-announcer"` |
| Toast      | `data-vize-ui="toast"`, `part="root"`, `data-state`, `data-type`, `data-priority`, `data-presence`, `data-swipe`, `data-swipe-direction`        |
| Swipe vars | `--vize-toast-swipe-move-x`, `--vize-toast-swipe-move-y` while dragging; `--vize-toast-swipe-end-x`, `--vize-toast-swipe-end-y` on dismiss      |
| Title/desc | `data-vize-ui="toast-title"` / `toast-description`, deterministic ids                                                                           |
| Action     | native button, `data-vize-ui="toast-action"`, `data-alt-text`                                                                                   |
| Close      | native button, `data-vize-ui="toast-close"`, `aria-label` default `"Dismiss notification"`                                                      |

Toast ships no stylesheet; the announcer carries inline visually-hidden styles only.
