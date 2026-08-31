# CopyButton Behavior

Normative state × input → outcome table for `copy-button.vue`.

| ID  | Scenario              | Behavior                                                                                   | Assertion                                                           |
| --- | --------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| C1  | native render         | renders a native `button` with `type="button"`, `part="root"`, label part, and idle state  | `renders deterministic native button semantics and default label`   |
| C2  | default clipboard     | activation writes the string through `navigator.clipboard.writeText` when available        | `uses navigator.clipboard.writeText by default`                     |
| C3  | successful write      | accepts the configured writer, emits `copy`, and exposes `data-state="copied"`             | `copies the configured value and exposes copied state`              |
| C4  | rejected write        | catches writer failures, emits `error`, and exposes `data-state="error"`                   | `captures writer failures without throwing out of activation`       |
| C5  | disabled              | native disabled removes activation; non-native disabled leaves tab order with ARIA state   | `disabled copy buttons suppress writes and keep platform semantics` |
| C6  | labels and slots      | label props and default slot receive the same strict `idle \| copied \| error` state       | `supports custom labels and slot rendering`                         |
| C7  | duplicate activation  | a write in flight sets busy hooks and suppresses additional accidental writes              | `suppresses duplicate writes while a copy is in flight`             |
| C8  | non-native activation | non-native hosts expose button semantics and emulate native Enter/Space timing             | `non-native hosts preserve keyboard button activation`              |
| C9  | public instance       | exposes live state, writing, unavailable, value, element, label, and `focus()`             | `exposes live state and focus without broad clipboard abstractions` |
| C10 | SSR and hydration     | setup never touches browser globals; server markup is stable and hydrates without warnings | `copy-button-ssr.test.ts` and runtime conformance fixtures          |

## Contract

CopyButton is a headless action primitive for copying one plain string value.
It intentionally owns only a narrow clipboard write: the default writer calls
`navigator.clipboard.writeText(value)` at activation time, and tests or SSR
consumers may inject a `writer` prop. Component setup, SSR rendering, and
hydration do not access `navigator`, `window`, timers, or document globals.

The result state is closed to `idle`, `copied`, and `error`. Disabled and
in-flight writes are separate availability hooks through `data-disabled`,
`data-writing`, `aria-disabled`, `aria-busy`, slot state, and the public expose
contract. In-flight writes preserve focus rather than applying native disabled.

The rendered DOM exposes `part="root"` on the host and `part="label"` on the
fallback label span. Stable selectors are `data-vize-ui="copy-button"` and
`data-vize-ui="copy-button-label"`. No CSS classes, runtime styles, CSS
custom properties, runtime CSS-in-JS, or copied value data attributes are
emitted; all visual styling remains consumer-owned.
