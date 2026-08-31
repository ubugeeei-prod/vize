# FullscreenButton Behavior

Normative state and input outcome table for `fullscreen-button.vue`.

| ID  | Scenario              | Behavior                                                                                                  | Assertion                                                                  |
| --- | --------------------- | --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| F1  | native render         | renders a native `button` with `type="button"`, `part="root"`, label part, idle state, and no styling     | `renders deterministic native button semantics and default label`          |
| F2  | non-native activation | non-native hosts expose button semantics and emulate native Enter/Space timing                            | `non-native hosts preserve keyboard button activation`                     |
| F3  | injected controller   | activation requests fullscreen for the captured target, then exits through the same controller model      | `runs the injected controller for enter and exit`                          |
| F4  | disabled              | native and non-native disabled states suppress activation without emitting fullscreen lifecycle events    | `disabled fullscreen buttons suppress actions and keep availability hooks` |
| F5  | duplicate activation  | entering and exiting states set busy hooks and suppress duplicate operations                              | `suppresses duplicate operations while entering and exiting are in flight` |
| F6  | rejected operation    | catches controller failures, emits `error` with the submitted operation, and exposes `data-state="error"` | `captures fullscreen failures without throwing out of activation`          |
| F7  | submitted controller  | an in-flight operation completes against the controller and target captured at activation time            | `uses the submitted controller when props change while entering`           |
| F8  | labels and slots      | label props and default slot receive the same strict `idle \| entering \| active \| exiting \| error`     | `supports custom labels and slot rendering`                                |
| F9  | public instance       | exposes live state, active, pending, unavailable, element, label, operation, and `focus()`                | `exposes live state and focus`                                             |
| F10 | SSR and hydration     | setup never touches browser globals or fullscreen APIs; markup is stable and hydrates without warnings    | `fullscreen-button-ssr.test.ts` and runtime conformance fixtures           |

## Contract

FullscreenButton is a headless action primitive for toggling the Fullscreen API.
The default controller resolves `document.documentElement` only when the user
activates the control, and the controller may be injected for tests, SSR, or
product-specific behavior. Component setup, server rendering, and hydration do
not touch document, window, timers, or fullscreen APIs.

The lifecycle state is closed to `idle`, `entering`, `active`, `exiting`, and
`error`. Pending operations preserve the submitted controller, target, and
operation payload across async prop changes. Duplicate activation is suppressed
while entering or exiting, but the rendered control remains focusable unless it
is explicitly disabled.

The rendered DOM exposes `part="root"` on the host and `part="label"` on the
fallback label span. Stable selectors are `data-vize-ui="fullscreen-button"` and
`data-vize-ui="fullscreen-button-label"`. Styling hooks are data attributes and
ARIA state only: `data-state`, `data-active`, `data-pending`, `data-disabled`,
`aria-pressed`, `aria-busy`, and `aria-disabled`. No CSS classes, runtime
styles, CSS custom properties, or action payload data attributes are emitted;
all visual styling remains consumer-owned.
