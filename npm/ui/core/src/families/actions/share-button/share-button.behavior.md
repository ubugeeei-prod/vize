# ShareButton Behavior

Normative state and input outcome table for `share-button.vue`.

| ID  | Scenario              | Behavior                                                                                              | Assertion                                                             |
| --- | --------------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| S1  | native render         | renders a native `button` with `type="button"`, `part="root"`, label part, idle state, and no styling | `renders deterministic native button semantics and default label`     |
| S2  | non-native activation | non-native hosts expose button semantics and emulate native Enter/Space timing                        | `non-native hosts preserve keyboard button activation`                |
| S3  | default share         | activation calls `navigator.share` with the normalized submitted payload when available               | `uses navigator.share by default`                                     |
| S4  | unavailable platform  | missing Web Share support emits a stable unavailable error payload without throwing from activation   | `emits a stable error when navigator.share is unavailable`            |
| S5  | injected action       | activation runs the captured action with title, text, url, files, and the triggering mouse event      | `runs the configured action and exposes shared state`                 |
| S6  | rejected action       | catches action failures, emits `error` with the submitted payload, and exposes `data-state="error"`   | `captures action failures without throwing out of activation`         |
| S7  | disabled              | native and non-native disabled states suppress activation without emitting share lifecycle events     | `disabled share buttons suppress actions and keep availability hooks` |
| S8  | duplicate activation  | sharing state sets busy hooks and suppresses duplicate actions                                        | `suppresses duplicate actions while sharing is in flight`             |
| S9  | submitted payload     | an in-flight operation completes against the action and payload captured at activation time           | `uses the submitted action and payload when props change`             |
| S10 | labels and slots      | label props and default slot receive the same strict `idle \| sharing \| shared \| error` state       | `supports custom labels and slot rendering`                           |
| S11 | public instance       | exposes live state, sharing, unavailable, payload, element, label, and `focus()`                      | `exposes live state and focus`                                        |
| S12 | SSR and hydration     | setup never touches browser globals or Web Share APIs; markup is stable and hydrates without warnings | `share-button-ssr.test.ts` and runtime conformance fixtures           |

## Contract

ShareButton is a headless action primitive for Web Share payloads. The default
action calls `navigator.share(payload)` only after the user activates the
control, and the action may be injected for tests, SSR, or product-specific
behavior. Component setup, server rendering, and hydration do not touch
`navigator`, `window`, `document`, timers, or Web Share APIs.

The lifecycle state is closed to `idle`, `sharing`, `shared`, and `error`.
The submitted action and normalized payload are captured before the async
operation starts, so prop changes while sharing do not affect completion emits.
Duplicate activation is suppressed while sharing, but the rendered control
remains focusable unless it is explicitly disabled.

The rendered DOM exposes `part="root"` on the host and `part="label"` on the
fallback label span. Stable selectors are `data-vize-ui="share-button"` and
`data-vize-ui="share-button-label"`. Styling hooks are data attributes and
ARIA state only: `data-state`, `data-sharing`, `data-disabled`, `aria-busy`,
and `aria-disabled`. No CSS classes, runtime styles, CSS custom properties, or
share payload data attributes are emitted; all visual styling remains
consumer-owned.
