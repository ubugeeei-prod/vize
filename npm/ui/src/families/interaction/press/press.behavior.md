# Press behavior contract

Normative state × input → outcome table for `@vizejs/ui/press`. Every row is
exercised by `src/families/interaction/press/press*.test.ts`;
compile-only API assertions live in
`src/families/interaction/press/press.types.test-d.ts`.

| #   | State                         | Input                                  | Outcome                                                                  | Proven by                                                      |
| --- | ----------------------------- | -------------------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------------------- |
| P1  | idle                          | primary pointer down                   | pressed state starts with the normalized device family                   | `normalizes a primary pointer lifecycle…`                      |
| P2  | idle                          | secondary or non-primary pointer       | input is ignored                                                         | `ignores non-primary contacts and secondary buttons`           |
| P3  | active pointer                | another pointer down                   | original pointer retains exclusive ownership                             | `ignores non-primary contacts and secondary buttons`           |
| P4  | active, inside                | primary pointer release then click     | up, end, and exactly one press are delivered                             | `normalizes a primary pointer lifecycle…`                      |
| P5  | active, inside                | pointer leaves                         | pressed state pauses with a canceled end snapshot                        | `pauses outside, resumes inside…`                              |
| P6  | paused outside                | owning pointer re-enters               | pressed state resumes with another start                                 | `pauses outside, resumes inside…`                              |
| P7  | paused outside                | owning pointer releases                | interaction ends without activation                                      | `an outside release cancels the default resumable interaction` |
| P8  | cancel-on-exit                | first pointer exit                     | interaction ends permanently and compatibility click is canceled         | `cancel-on-exit suppresses the following compatibility click`  |
| P9  | active                        | pointer cancel, drag, blur, or hidden  | state and transient resources are canceled                               | lifecycle listener and cancellation tests                      |
| P10 | active                        | reactive disabled becomes true         | release cancels; subsequent click default is prevented                   | `cancels when disabled changes during a press`                 |
| P11 | disabled                      | pointer, keyboard, or click            | no callback is emitted; click default is prevented                       | disabled and reactive-option tests                             |
| P12 | idle custom button            | Enter                                  | start/up/end/press lifecycle is synthesized                              | `emulates button and link keyboard semantics…`                 |
| P13 | idle custom button            | Space                                  | scrolling is prevented; activation occurs on keyup                       | `emulates button and link keyboard semantics…`                 |
| P14 | idle custom link              | Space / Enter                          | Space is ignored; Enter activates                                        | `emulates button and link keyboard semantics…`                 |
| P15 | native activatable element    | browser keyboard click                 | native timing/default action wins; one normalized press is reported      | `preserves native keyboard click timing…`                      |
| P16 | keyboard interaction          | IME, repeat, or nested interactive key | event is ignored and no ancestor activation is invented                  | `ignores IME, repeats, nested targets…`                        |
| P17 | idle                          | coordinate-free click                  | complete lifecycle is reported as `virtual`                              | `maps coordinate-free and click-only activation…`              |
| P18 | idle                          | click with coordinates/detail          | complete lifecycle is reported as `mouse`                                | `maps coordinate-free and click-only activation…`              |
| P19 | active pointer                | selection guard                        | exact inline user-select values and priorities restore at end            | `restores exact selection styles…`                             |
| P20 | prevent-focus option          | compatibility mousedown                | default focus transfer is prevented without suppressing keyboard focus   | `restores exact selection styles…`                             |
| P21 | callback re-enters controller | start callback cancels                 | stale `pressed=true` change cannot publish after the cancellation        | `a reentrant cancel cannot publish…`                           |
| P22 | multiple callbacks throw      | coordinate-free activation             | all lifecycle phases run, state settles, then errors surface             | `synthetic activation completes every phase…`                  |
| P23 | active callback throws        | explicit cancellation                  | interaction remains owned and can still release every listener           | `callback failure leaves an active interaction…`               |
| P24 | Vue effect scope              | scope stops                            | controller disposes, state resets, listeners and timers release          | `the composable requires and follows…`                         |
| P25 | disposed                      | late renderer event / explicit cancel  | bound handlers are inert; imperative mutation throws a stable diagnostic | disposal lifecycle tests                                       |
| P26 | concurrent SSR requests       | identical component trees              | byte-identical markup contains no serialized handler or global DOM read  | `renders byte-identical SSR output…`                           |
| P27 | SSR followed by hydration     | coordinate-free client activation      | root identity is retained; no diagnostics; callback updates render       | `hydrates without diagnostics…`                                |
| P28 | public TypeScript API         | mutation or invalid closed-union use   | compilation rejects the misuse                                           | `src/families/interaction/press/press.types.test-d.ts`         |
| P29 | template consumer             | native DOM / SSR / Vapor compilation   | handler spread and reactive state compile with no warning or fallback    | `scripts/check-renderers.ts` press fixture                     |

## Accessibility and native behavior

- `click` is the activation authority. Pointer release produces `pressup` and
  `pressend`, while the browser's following click produces `press`. This keeps
  link navigation, form submission, label forwarding, and assistive-technology
  activation intact without double invocation.
- Native buttons, relevant input types, links with `href`, and summaries retain
  browser keyboard timing and default actions. Custom button-like hosts receive
  Enter and Space behavior; custom link-like hosts receive Enter only. An `<a>`
  or `<area>` without `href` has no native activation, so it follows
  `keyboardBehavior` like any other custom host.
- Space scrolling is canceled only for custom button semantics. The primitive
  does not cancel native key events whose default action creates the click.
- Disabled state is evaluated at every phase. If it changes during an active
  gesture, the gesture and its compatibility click are both canceled.
- Events from focused descendants never invent an activation on the bound
  ancestor. Composite widgets remain responsible for their own key routing.

## Pointer ownership and cancellation

- Only the primary contact and primary button can own a press. An active
  interaction ignores additional contacts until it ends.
- Pointer Events are preferred. Legacy mouse and touch handlers are present for
  older engines, including the compatibility-mouse suppression window after a
  touch contact.
- Leaving the target pauses pressed feedback and re-entry resumes it. Set
  `shouldCancelOnPointerExit` to make the first exit terminal.
- Window blur, hidden documents, pointer cancellation, drag start, manual
  cancellation, scope disposal, and disabled transitions all release document
  listeners and restore transient text-selection state. Only a blur of the window
  itself cancels; focus moving between elements during a press does not.
- `preventFocusOnPress` suppresses the `mousedown` focus default only for an
  enabled host and a primary button, including in Pointer Events engines where
  `mousedown` still owns that default.

## SSR, Vapor, styling, and tree shaking

- Module evaluation and `createPress()` perform no DOM read, native listener
  installation, timer scheduling, style injection, or request-global mutation.
  Listeners and timers exist only during a client interaction.
- `usePress()` only requires an active Vue effect scope; its plain handler props
  are compatible with DOM rendering, standard SSR/hydration, and Vapor-compiled
  consumers. No renderer-specific VNode or component instance is captured.
- This entry emits zero CSS and no attributes. Consumers map readonly
  `isPressed` to any classes, data attributes, utility system, CSS variables, or
  preset package they choose.
- `@vizejs/ui/press` is an exact ESM subpath. The production consumer gate
  requires root/subpath byte parity, zero CSS, unused-family elimination, and a
  hard gzip budget.

## Normative references

- [Pointer Events Level 4 — primary pointer, buttons, capture, and click](https://www.w3.org/TR/pointerevents/)
- [UI Events — accessible activation through click](https://www.w3.org/TR/uievents/)
- [WAI-ARIA APG — Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/)
- [WAI-ARIA APG — Link Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/link/)
- [WAI-ARIA APG — Developing a Keyboard Interface](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)
