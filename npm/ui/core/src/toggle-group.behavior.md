Normative contract for `toggle-group.vue` and `toggle-group-item.vue` (`@vizejs/ui/toggle-group`).
Every row is proven by the named mounted-DOM, SSR, type, renderer, size, and tree-shaking gates.

| ID  | State                        | Input                  | Outcome                                                                                               | Evidence                                                            |
| --- | ---------------------------- | ---------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| G1  | single, empty                | render                 | root renders `role="group"` and horizontal data hooks; items render native buttons with pressed state | `renders grouped toggle button semantics`                           |
| G2  | uncontrolled single          | item click             | pressing an unpressed item selects its value; pressing it again clears the single value               | `uncontrolled single mode toggles one value and emits changes`      |
| G3  | controlled single            | item click             | emits the requested next value while DOM state stays controlled until the parent accepts it           | `controlled single value wins until the parent accepts the request` |
| G4  | uncontrolled multiple        | item click             | pressed values are appended and removed as an immutable array without duplicates                      | `multiple mode adds and removes item values`                        |
| G5  | roving horizontal/vertical   | Arrow/Home/End keydown | focus moves through enabled items, respects orientation, skips disabled items, and honors loop        | `roving focus follows orientation and skips disabled items`         |
| G6  | disabled group or item       | click or keyboard      | activation is suppressed; native buttons receive `disabled`, custom hosts receive `aria-disabled`     | `disabled groups and items suppress activation`                     |
| G7  | item without provider        | setup                  | item setup throws the shared missing-context diagnostic                                               | `items require a matching group provider`                           |
| G8  | SSR and hydration            | isolated requests      | renders byte-identical markup and hydrates without diagnostics or node replacement                    | `src/toggle-group-ssr.test.ts`                                      |
| G9  | DOM/SSR/Vapor                | compile                | authored root, item, and consumer SFCs compile in every renderer lane without fallback                | `scripts/check-renderers.ts`                                        |
| G10 | root/subpath consumer bundle | production build       | root and subpath imports retain only the toggle-group contract, emit no CSS, and stay in budget       | `scripts/check-tree-shaking.mjs`                                    |

| Surface             | Contract                                                                    | Default        |
| ------------------- | --------------------------------------------------------------------------- | -------------- |
| `type`              | `"single" \| "multiple"` controls item toggle behavior                      | `"single"`     |
| `modelValue`        | `string \| readonly string[] \| null`; `undefined` selects uncontrolled use | `undefined`    |
| `defaultValue`      | initial uncontrolled value normalized by `type`                             | `null` or `[]` |
| `orientation`       | `"horizontal" \| "vertical"` exposed to data hooks and arrow navigation     | `"horizontal"` |
| `loop`              | wraps roving focus at the first and last enabled item                       | `true`         |
| `rovingFocus`       | keeps enabled items in a single-tabstop focus model                         | `true`         |
| item `value`        | string membership key for pressed state                                     | required       |
| `data-vize-ui`      | `"toggle-group"` on root and `"toggle-group-item"` on items                 | always         |
| item `aria-pressed` | `"true"` while the item value is selected, otherwise `"false"`              | always         |
