# Long-press behavior contract

Normative state × input → outcome table for `@vizejs/ui/long-press`. Every row is
exercised by `src/families/interaction/long-press/long-press*.test.ts`;
compile-only assertions live in
`src/families/interaction/long-press/long-press.types.test-d.ts`.

| #    | State                          | Input                                              | Outcome                                                                   | Proven by                                                        |
| ---- | ------------------------------ | -------------------------------------------------- | ------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| L1   | idle                           | primary mouse, pen, touch, or unknown pointer down | start snapshot and pressed state are published                            | stable lifecycle and pointer-filter tests                        |
| L2   | idle                           | secondary or non-primary pointer                   | input is ignored                                                          | secondary-contact test                                           |
| L3   | pending                        | threshold elapses                                  | exactly one long event is emitted; short activation is suppressed         | stable lifecycle test                                            |
| L4   | pending                        | primary release before threshold                   | long end without a long event and ordinary short press are emitted        | short-alternative test                                           |
| L5   | pending                        | pointer leaves host                                | timer and attempt cancel permanently                                      | cancellation test                                                |
| L6   | pending or triggered           | reactive disabled becomes true                     | threshold/release cancels or marks the terminal event canceled            | cancellation test                                                |
| L7   | triggered                      | owning pointer releases                            | normal long end is emitted and state settles                              | stable lifecycle test                                            |
| L8   | triggered                      | another contact starts or releases                 | original contact retains ownership in either release order                | contact-ownership and legacy touch tests                         |
| L9   | pending or triggered           | manual cancel                                      | canceled end is emitted and compatibility click is suppressed             | cancellation tests                                               |
| L10  | active                         | dispose, scope stop, or cleanup callback failure   | every teardown is attempted and the controller becomes terminal           | disposal/scope and cleanup-failure tests                         |
| L11  | touch or pen attempt           | native context menu                                | active and immediately trailing menus are prevented                       | context-menu test                                                |
| L12  | mouse or idle                  | native context menu                                | browser default remains untouched                                         | context-menu test                                                |
| L13  | active                         | text-selection guard                               | exact inline values and priorities restore at end                         | selection test                                                   |
| L13a | touch or pen reaches threshold | host is not focused                                | host receives focus without requesting scroll movement                    | selection/context-menu test                                      |
| L14  | configured pointer filter      | another device family                              | long recognition is skipped                                               | pointer-filter test                                              |
| L15  | reactive options               | next attempt/render                                | current threshold, filter, disabled state, and description are resolved   | reactive-options test                                            |
| L16  | accessible description ID      | render                                             | `aria-describedby` wins over inline description                           | reactive-options test                                            |
| L17  | inline accessible description  | render                                             | `aria-description` is exposed only for enabled long actions               | reactive-options test                                            |
| L18  | keyboard or virtual activation | press                                              | explicit short/alternative callback remains available                     | short-alternative test                                           |
| L19  | legacy Touch Events browser    | multi-touch owning release                         | coordinates select the owning identifier even when another touch is first | legacy touch test                                                |
| L20  | invalid JavaScript options     | setup or triggered release                         | stable diagnostics reject misuse after terminal state has settled         | runtime-option and teardown tests                                |
| L21  | public TypeScript API          | mutation or invalid unions                         | compilation rejects misuse                                                | `src/families/interaction/long-press/long-press.types.test-d.ts` |

## Accessibility obligations

Long press is a pointing gesture, not a complete keyboard command. Consumers
must expose the same action through `onPress`, another visible control, or a
documented keyboard command. The controller supplies a reactive accessible
description and never invents a keyboard gesture whose meaning depends on the
owning component. Context-menu suppression is limited to touch and pen attempts;
ordinary mouse context menus retain native behavior.

The interaction follows Pointer Events cancellation and containment semantics,
preserves native keyboard activation through the Press foundation, and requires
no CSS. Consumer styling can read `isPressed` and `isLongPressed` without a
class-name, DOM-shape, or theme contract.
