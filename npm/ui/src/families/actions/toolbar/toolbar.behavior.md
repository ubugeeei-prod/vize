# Toolbar Behavior

Every row is proven by a mounted-DOM, SSR/hydration, renderer, type, size, or
tree-shaking gate. A row without a passing test is a bug in the family contract.

| ID  | Component          | Scenario          | Behavior                                                                                    | Assertion                                                             |
| --- | ------------------ | ----------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| T1  | `toolbar.vue`      | semantic root     | renders `role="toolbar"` with labeling, orientation, direction, part, data, and style hooks | `renders accessible toolbar semantics without adding visual CSS`      |
| T2  | `toolbar-item.vue` | vertical roving   | one-tabstop arrow-key focus follows orientation and skips disabled native buttons           | `roving focus follows vertical orientation and skips disabled items`  |
| T3  | `toolbar.vue`      | native tab order  | `rovingFocus=false` preserves native tab order and lets arrow keys pass through             | `roving focus can be disabled to preserve the native tab order`       |
| T4  | `toolbar-item.vue` | rtl navigation    | horizontal arrow-key focus maps next and previous through the toolbar `dir` contract        | `horizontal roving focus respects rtl direction`                      |
| T5  | `toolbar-item.vue` | activation        | enabled items emit item and toolbar `press` events in dispatch order                        | `pointer and keyboard activation emit value-carrying events`          |
| T6  | both               | disabled          | disabled toolbars and items suppress activation and leave tab order safely                  | `disabled toolbars and items suppress activation`                     |
| T7  | `toolbar-item.vue` | custom element    | non-native items receive `role="button"` and synthesize Enter/Space keyboard clicks         | `custom items expose button semantics and keyboard activation`        |
| T8  | both               | public instance   | root and items expose focus methods and immutable state snapshots                           | `exposes focus, focusValue, activeValue, and live state`              |
| T9  | `toolbar-item.vue` | duplicate value   | mount and reactive value changes reject duplicate item values                               | `rejects duplicate item values before roving focus becomes ambiguous` |
| T10 | `toolbar-item.vue` | provider boundary | items fail loudly when rendered outside a matching toolbar provider                         | `items require a matching toolbar provider`                           |
| T11 | both               | SSR/hydration     | server markup is deterministic and hydrates without replacing the root or drifting tabindex | `toolbar-ssr.test.ts`, `runtime-conformance.test.ts`                  |
| T12 | public types       | invalid contract  | TypeScript rejects unsupported orientation, direction, native button type, and item values  | `src/families/actions/toolbar/toolbar.types.test-d.ts`                |
| T13 | root/subpath       | consumer bundle   | root and subpath consumers retain only Toolbar, emit no CSS, and stay within gzip budget    | `scripts/check-tree-shaking.mjs`                                      |

## Contract

Toolbar is a headless compound primitive for clustered actions such as editor
controls, document commands, and inspector panels. It owns no selection model and
emits no visual CSS. Consumers style through `data-vize-ui`, `data-state`,
`data-disabled`, `data-orientation`, `data-roving-focus`, `data-value`, `part`,
and `--vize-ui-toolbar-orientation`.

The root always renders `role="toolbar"`. Items are native `<button
type="button">` controls by default, and non-native targets receive explicit
button semantics plus Enter/Space activation. Roving focus is enabled by default
so the toolbar has a single tab stop; consumers can set `rovingFocus=false` when
native per-control tab stops are required.

Items require a unique string `value` so item-level and toolbar-level `press`
events, `activeValue`, roving tabindex, and `focusValue()` stay stable across
polymorphic rendering, slot changes, and DOM reordering. Duplicate values throw
`VIZE_UI_TOOLBAR_VALUE_DUPLICATE` during registration or reactive value changes
before more than one item can own the active tab stop.
