# ButtonGroup Behavior

| ID  | Component               | Scenario                  | Behavior                                                                 | Assertion                                                      |
| --- | ----------------------- | ------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------------------- |
| B1  | `button-group.vue`      | grouped actions           | renders an accessible `group` with item data, part, and disabled hooks   | `renders grouped button semantics without adding visual CSS`   |
| B2  | `button-group.vue`      | toolbar navigation        | `role="toolbar"` opts into one-tabstop roving focus by default           | `toolbar roving focus follows orientation and skips disabled`  |
| B3  | `button-group.vue`      | group navigation override | plain groups may opt into roving focus while preserving their ARIA role  | `plain groups can opt into roving focus without toolbar role`  |
| B4  | `button-group-item.vue` | activation                | enabled items emit item and group `press` events in dispatch order       | `pointer and keyboard activation emit value-carrying events`   |
| B5  | `button-group-item.vue` | disabled                  | disabled groups and items suppress activation and leave tab order safely | `disabled groups and items suppress activation`                |
| B6  | `button-group-item.vue` | custom element semantics  | non-native items receive `role="button"` and synthesize keyboard clicks  | `custom items expose button semantics and keyboard activation` |
| B7  | both                    | public instance           | root and items expose focus methods and immutable state snapshots        | `exposes focus, focusValue, activeValue, and item state`       |
| B8  | `button-group-item.vue` | provider boundary         | items fail loudly when rendered outside a matching group provider        | `items require a matching group provider`                      |

## Contract

ButtonGroup is a headless compound primitive for adjacent actions such as
button bars, editor toolbars, card action rows, and destructive-confirmation
clusters. It owns no selection model and emits no visual CSS. Consumers style
through `data-vize-ui`, `data-state`, `data-disabled`, `data-orientation`,
`data-role`, and `part` attributes.

`role="group"` preserves the native tab order by default. `role="toolbar"`
defaults to a single tab stop with arrow-key movement because that is the WAI-ARIA
toolbar expectation. Consumers may explicitly set `rovingFocus` on either role.

Items require a string `value` so item-level and group-level `press` events are
stable across polymorphic rendering, slot changes, and DOM reordering.
