# Interaction modality behavior contract

Normative state × input → outcome table for `@vizejs/ui/interaction-modality`.
Every row is exercised by `src/families/accessibility/interaction-modality/interaction-modality*.test.ts`;
compile-only API assertions live in
`src/families/accessibility/interaction-modality/interaction-modality.types.test-d.ts`.

| #    | State                               | Input                               | Outcome                                                   | Proven by                                                                              |
| ---- | ----------------------------------- | ----------------------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| IM1  | no DOM / SSR                        | create with `document: null`        | no native access; reactive state remains usable           | `is inert without a document and remains manually controllable…`                       |
| IM2  | any                                 | unmodified keyboard intent          | modality becomes `keyboard`; focus is globally visible    | `classifies keyboard, pointer, touch, and virtual intent`                              |
| IM3  | any                                 | mouse, pen, or unknown pointer      | modality becomes `pointer`                                | `classifies keyboard, pointer, touch, and virtual intent`                              |
| IM4  | any                                 | touch pointer                       | modality becomes `touch`                                  | `classifies keyboard, pointer, touch, and virtual intent`                              |
| IM5  | any except keyboard                 | coordinate-free click               | modality becomes `virtual`                                | `keeps keyboard-generated coordinate-free clicks…`                                     |
| IM6  | keyboard                            | synthesized click with `detail: 0`  | keyboard modality is retained                             | `keeps keyboard-generated coordinate-free clicks…`                                     |
| IM7  | any                                 | IME or modified shortcut            | event does not alter modality                             | `ignores composition, modifier-only keys, and modified shortcuts`                      |
| IM8  | two trackers in one document        | input or manual update              | one listener set; both trackers receive identical state   | `shares exactly one native listener set and state per document`                        |
| IM9  | trackers in different documents     | input in one document               | other document remains unchanged                          | `isolates separate documents and adopts existing state when moving`                    |
| IM10 | tracker moves to populated document | reactive document change            | existing document state is adopted with `document` reason | `isolates separate documents and adopts existing state when moving`                    |
| IM11 | attached                            | detach                              | listeners release; last modality remains                  | `supports explicit attach and detach without losing the last value`                    |
| IM12 | disposed                            | attach, detach, or state mutation   | stable disposed diagnostic is thrown                      | `disposal is idempotent and rejects later mutation`                                    |
| IM13 | Vue effect scope                    | scope stops                         | tracker disposes and releases its shared subscription     | `the composable requires and follows a Vue effect scope`                               |
| IM14 | focused element, supported browser  | visibility query                    | native `:focus-visible` result wins                       | `defers to native focus-visible semantics…`                                            |
| IM15 | focused element, old browser        | selector throws                     | keyboard/virtual modality supplies the fallback           | `falls back to modality only when focus-visible is unsupported`                        |
| IM16 | unfocused element                   | visibility query                    | result is always false                                    | `defers to native focus-visible semantics…`                                            |
| IM17 | public TypeScript API               | invalid modality or mutable ref use | compilation rejects the misuse                            | `src/families/accessibility/interaction-modality/interaction-modality.types.test-d.ts` |
| IM18 | legacy touch browser                | touch then compatibility mousedown  | touch modality is retained                                | `shares exactly one native listener set and state per document`                        |
| IM19 | subscriber callback                 | reentrant modality change           | queued delivery leaves every live peer in the same state  | `serializes reentrant updates so every peer reaches…`                                  |
| IM20 | subscriber callback                 | callback throws                     | all peers update before the exception is surfaced         | `updates every peer before surfacing a subscriber exception`                           |
| IM21 | document adoption                   | synchronization callback throws     | new subscription rolls back without a listener leak       | `rolls back a failed document adoption without leaking…`                               |
| IM22 | concurrent SSR requests             | identical component trees           | byte-identical, detached, neutral output is rendered      | `renders byte-identical output without touching a server document`                     |
| IM23 | SSR followed by hydration           | document becomes available on mount | listener attaches after warning-free hydration            | `attaches after hydration without mismatch diagnostics`                                |

## Accessibility decisions

- Native `:focus-visible` is authoritative when available. This preserves
  browser heuristics for editable controls, platform conventions, and assistive
  technology instead of replacing them with a narrower JavaScript guess.
- `keyboard` and `virtual` are the only globally focus-visible modalities.
  Components should use `isElementFocusVisible` for the final per-element
  decision and must still provide an actual visible style.
- Pointer Events reserve `pointerId: -1` for input not produced by pointing
  hardware. An empty-type pointer with that ID is classified as `virtual`.
- Modified shortcuts and composition do not change visual modality. Shift+Tab
  remains keyboard intent because it directly changes focus.

## SSR, hydration, and document ownership

- Importing this entry performs no DOM read, listener installation, style
  injection, timer scheduling, or mutation of request-scoped state.
- With no global document, the default tracker is inert and deterministic.
- Trackers share listeners only when keyed by the same live `Document`. Weak
  ownership and reference counting release the hub when its final subscriber
  disposes; iframes and independently hydrated documents cannot leak state.
- Pass a reactive document getter when an island, iframe, or portal owner is not
  known until mount. Passing `null` explicitly is the SSR/deferred escape hatch.

## Styling contract

This primitive emits no CSS and sets no attributes. Consumers may map
`isFocusVisible` or `isElementFocusVisible` to classes, data attributes, CSS
variables, utility frameworks, or fully custom styles. Never remove the browser
outline unless an equivalent visible indicator is supplied.

## Normative references

- [WCAG 2.2 — Focus Visible (2.4.7)](https://www.w3.org/WAI/WCAG22/Understanding/focus-visible)
- [Selectors Level 4 — `:focus-visible`](https://www.w3.org/TR/selectors-4/#the-focus-visible-pseudo)
- [Pointer Events — pointer types and non-pointing input](https://w3c.github.io/pointerevents/)
