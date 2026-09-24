# Variants behavior contract

Normative state x input -> outcome table for `@vizejs/ui/variants`. Every row is
proven by `src/families/foundations/variants/variants.test.ts`; compile-only
inference assertions live in `src/families/foundations/variants/variants.types.test-d.ts`.

| #    | State                  | Input                             | Outcome                                                                                                          | Proven by                                                                                        |
| ---- | ---------------------- | --------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| VR1  | any class values       | `cx(...)`                         | strings/numbers/bigints joined in order; nested lists flattened; truthy dictionary keys kept                     | `cx joins strings, numbers, nested lists, and truthy dictionary keys`                            |
| VR2  | merge hook             | `createCx(merge)(...)`            | joined output is passed through the hook exactly once                                                            | `createCx applies the merge hook to the joined output`                                           |
| VR3  | class recipe           | call with and without props       | base, defaults, variant options, matching compounds, then `class`, in that order                                 | `recipes apply base, defaults, variants, compounds, then class in order`                         |
| VR4  | unknown option / unset | runtime props outside the types   | unknown options are ignored; an unset variant with a `false` option uses it                                      | `unknown runtime options are ignored and undefined boolean variants use false`                   |
| VR5  | recipe merge option    | call                              | the configured merger post-processes recipe output                                                               | `merge hook post-processes recipe output`                                                        |
| VR6  | slot recipe            | call, then slot functions         | one function per slot (`base` first); per-slot options/compounds; slot overrides append                          | `slot recipes return per-slot class functions with overrides`                                    |
| VR7  | responsive breakpoints | `{ initial, md }` value           | `initial` classes unprefixed, others prefixed per token; unknown breakpoints throw `VIZE_UI_VARIANTS_BREAKPOINT` | `responsive values prefix classes per breakpoint and reject unknown breakpoints`                 |
| VR8  | forwarding             | `splitVariantProps(props)`        | variant props and the remaining props are separated                                                              | `splitVariantProps separates variant props from forwarded props`                                 |
| VR9  | server render          | render twice, then hydrate        | byte-identical markup and no hydration diagnostics                                                               | `renders identical classes on the server`, `hydrates server markup without mismatch diagnostics` |
| VR10 | public type API        | props, defaults, compounds, slots | option unions, boolean variants, slot keys, and breakpoints are inferred; typos fail to compile                  | `src/families/foundations/variants/variants.types.test-d.ts`                                     |

Recipes are pure functions of their input: no DOM, globals, or lifecycle, so
they are identical on the server, in Vapor components, and in plain scripts.
