# FocusVisible behavior contract

Normative behavior for `@vizejs/ui/focus-visible`. `focus-visible-provider.vue`
(`FocusVisibleProvider`) reuses the shared document interaction-modality tracker and publishes
uniform data attributes so CSS can style `[data-focus-visible]` identically across engines,
including engines whose `:focus-visible` heuristics differ. Every row is proven by the named test.

| State x input                                        | Observable outcome                                                                                                                                  | Proven by                                                                    |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| keyboard interaction, focus moves to a descendant    | The descendant gets `data-focus-visible="true"`, the root gets `data-vize-modality="keyboard"` and `data-focus-visible-within`.                     | `keyboard focus marks the focused descendant and publishes the modality`     |
| focus moves between descendants                      | The previous descendant loses the attribute and the new one gains it.                                                                               | `keyboard focus marks the focused descendant and publishes the modality`     |
| pointer press or pointer focus on a non-text control | The mark is removed (or never set) and the root reports `data-vize-modality="pointer"`.                                                             | `pointer focus on buttons is not marked and a pointer press clears the mark` |
| any modality focusing a text entry field             | Text inputs, textareas, and contenteditable hosts are marked, including after touch (`data-vize-modality="touch"`).                                 | `text entry fields are marked for every modality, touch included`            |
| focus leaves the subtree                             | The mark and `data-focus-visible-within` are removed.                                                                                               | `blur leaving the subtree removes the mark`                                  |
| `attribute` / `disabled`                             | The mark uses the configured attribute (renaming moves it); `disabled` removes every published attribute.                                           | `custom attribute names and disabled providers`                              |
| focus already inside the subtree when mounting       | The provider marks the active descendant on mount.                                                                                                  | `focus already inside the subtree at mount is picked up`                     |
| `useFocusVisible()`                                  | Inside a provider it reads the provider state; standalone it tracks the document modality; outside a scope it throws `VIZE_UI_FOCUS_VISIBLE_SETUP`. | `useFocusVisible reads the provider or tracks the document standalone`       |
| heuristics                                           | Keyboard and virtual focus, text entry, and unknown modality are indicated; pointer and touch on other controls are not.                            | `heuristics mirror browser focus-visible rules`                              |
| SSR                                                  | Server markup is byte-identical and carries no modality or focus attributes.                                                                        | `renders byte-identical markup without modality or focus attributes`         |
| hydration                                            | Hydration emits no warnings even when the document already has a modality; attributes appear after mount.                                           | `hydrates without diagnostics even when the document already has a modality` |
| public types                                         | Modality, state, and props are closed contracts.                                                                                                    | `src/families/accessibility/focus-visible/focus-visible.types.test-d.ts`     |
| DOM/SSR/Vapor                                        | `focus-visible-provider.vue` compiles in every renderer lane.                                                                                       | `scripts/check-renderers.ts`                                                 |

| Target     | Public contract                                                                                                            |
| ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| Root       | `part="root"`, `data-vize-ui="focus-visible-provider"`, `data-vize-modality`, `data-focus-visible-within`, `data-disabled` |
| Descendant | `data-focus-visible="true"` (or the configured `attribute`) while focus should be indicated                                |

FocusVisible ships no stylesheet.
