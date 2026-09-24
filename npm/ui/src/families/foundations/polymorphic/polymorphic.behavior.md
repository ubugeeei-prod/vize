# Polymorphic behavior contract

Normative state x input -> outcome table for `@vizejs/ui/polymorphic`. Every row
is proven by `src/families/foundations/polymorphic/polymorphic.test.ts`;
compile-only assertions live in
`src/families/foundations/polymorphic/polymorphic.types.test-d.ts`.

| #   | State               | Input                                    | Outcome                                                                                                                             | Proven by                                                                                       |
| --- | ------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| PM1 | tag or component    | `isPolymorphicTag(as)`                   | strings are tags; components are not                                                                                                | `distinguishes native tags from components`                                                     |
| PM2 | `as="button"`       | resolve attributes                       | `type="button"` is added unless a type is given                                                                                     | `buttons default to type=button while explicit types win`                                       |
| PM3 | disabled non-button | resolve attributes                       | anchors drop `href`, gain `aria-disabled` and `tabindex=-1`; others gain `aria-disabled`; empty tags throw `VIZE_UI_POLYMORPHIC_AS` | `disabled anchors and generic elements expose aria-disabled`                                    |
| PM4 | tags and components | `renderPolymorphic(as, attrs, children)` | native children or a default slot render in the DOM                                                                                 | `renders native tags and components with slot children in the DOM`                              |
| PM5 | server render       | render twice, then hydrate               | byte-identical markup and no hydration diagnostics                                                                                  | `renders identical markup on the server`, `hydrates server markup without mismatch diagnostics` |
| PM6 | public type API     | `as` plus attributes                     | native attributes / component props are inferred from `as`; wrong attributes fail                                                   | `src/families/foundations/polymorphic/polymorphic.types.test-d.ts`                              |

`Primitive` remains the SFC for template use; these helpers type the same `as`
contract for render functions and wrapper component props.
