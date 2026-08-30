# PrintButton Behavior

Normative state and input outcome table for `print-button.vue`.

| ID  | Scenario              | Behavior                                                                                          | Assertion                                                            |
| --- | --------------------- | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| P1  | native render         | renders a native `button` with `type="button"`, `part="root"`, label part, and idle state         | `renders deterministic native button semantics and default label`    |
| P2  | default print action  | activation calls the platform print function when available                                       | `uses the platform print function by default`                        |
| P3  | successful action     | accepts the configured action, emits `print`, and exposes `data-state="printed"`                  | `runs the configured action and exposes printed state`               |
| P4  | rejected action       | catches action failures, emits `error`, and exposes `data-state="error"`                          | `captures action failures without throwing out of activation`        |
| P5  | disabled              | native disabled removes activation; non-native disabled leaves tab order with ARIA state          | `disabled print buttons suppress action and keep platform semantics` |
| P6  | labels and slots      | label props and default slot receive the same strict `idle \| printing \| printed \| error` state | `supports custom labels and slot rendering`                          |
| P7  | duplicate activation  | a pending action sets busy hooks and suppresses additional accidental actions                     | `suppresses duplicate actions while printing is in flight`           |
| P8  | submitted action      | an in-flight action completes against the handler captured at activation time                     | `uses the submitted action when props change while printing`         |
| P9  | non-native activation | non-native hosts expose button semantics and emulate native Enter/Space timing                    | `non-native hosts preserve keyboard button activation`               |
| P10 | public instance       | exposes live state, printing, unavailable, element, label, and `focus()`                          | `exposes live state and focus`                                       |
| P11 | SSR and hydration     | setup never touches browser globals; server markup is stable and hydrates without warnings        | `print-button-ssr.test.ts` and runtime conformance fixtures          |

## Contract

PrintButton is a headless action primitive for invoking the user agent's print
flow. It intentionally owns only a narrow action boundary: the default action
calls the platform print function at activation time, and tests, SSR consumers,
or product integrations may inject an `action` prop. Component setup, SSR
rendering, and hydration do not access `navigator`, `window`, timers, or
document globals.

The lifecycle state is closed to `idle`, `printing`, `printed`, and `error`.
Disabled and in-flight actions are separate availability hooks through
`data-disabled`, `data-printing`, `aria-disabled`, `aria-busy`, slot state, and
the public expose contract. In-flight actions preserve focus rather than
applying native disabled.

The rendered DOM exposes `part="root"` on the host and `part="label"` on the
fallback label span. Stable selectors are `data-vize-ui="print-button"` and
`data-vize-ui="print-button-label"`. No CSS classes, runtime styles, CSS
custom properties, runtime CSS-in-JS, or action payload data attributes are
emitted; all visual styling remains consumer-owned.
