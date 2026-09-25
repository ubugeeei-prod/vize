# Cascader Behavior

## Contract

Cascader is a headless, strongly typed multi-level picker. `cascader-root.vue`
is generic over the node type `T` and a `Multiple` literal: the model is one
option path (`readonly T[]`, empty when nothing is selected) or, with
`multiple: true`, a list of leaf paths (`readonly (readonly T[])[]`). Children
come from `getChildren` (default: an array `children` property) or lazily from
`loadChildren(node, { signal })`; nodes compare through `by`.

`cascader-trigger.vue` renders a native `<button role="combobox"
aria-haspopup="listbox">` that keeps DOM focus and exposes the highlighted
option through `aria-activedescendant`; `aria-controls` lists every open
column. `cascader-content.vue` renders the popup through Portal, Presence,
Positioner, and a dismissable layer. The root publishes typed `columns` (the
top level plus one per expanded branch); each `cascader-column.vue` is a
`role="listbox"` labelled by its parent option and each `cascader-item.vue` a
`role="option"` with `aria-expanded` on branches. `cascader-value.vue` renders
the selected path text or a placeholder. Named cascaders submit one hidden
input per selected path (`formValue`/`by` segments joined by `separator`).
Option ids derive from the root id plus level and index. No CSS ships.

## Normative Behavior

| #   | State                 | Input                                                                                       | Outcome                                                                                                                                                                                                                 | Proven by                                                                                     |
| --- | --------------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| K1  | closed                | render                                                                                      | combobox trigger with `aria-haspopup="listbox"`, `aria-expanded="false"`, placeholder text, no columns                                                                                                                  | `renders a combobox trigger with placeholder and no popup while closed`                       |
| K2  | open                  | click branch / click leaf                                                                   | branches open child columns labelled by their parent and set `aria-expanded`; a leaf selects its path, closes, and shows `A / B / C`                                                                                    | `click expands branches into new columns and selecting a leaf closes`                         |
| K3  | selected              | reopen                                                                                      | expands the selected path, highlights its last node, marks ancestors `data-state="partial"`                                                                                                                             | `reopening restores the selected path and marks ancestors as partial`                         |
| K4  | closed / open         | ArrowDown / Up / Home / End / ArrowRight / Enter / ArrowLeft / Space                        | opens; moves within a column skipping disabled nodes; ArrowRight and Enter on a branch descend to its first option; ArrowLeft returns to the parent and closes deeper columns; Enter/Space on a leaf selects and closes | `keyboard matrix: open, arrows skip disabled, right/left traverse levels, Enter selects`      |
| K5  | open                  | printable keys / Escape                                                                     | typeahead per column (navigation keys reset the query); Escape closes without selecting                                                                                                                                 | `typeahead moves within the active column and Escape closes without selecting`                |
| K6  | `changeOnSelect`      | click branch                                                                                | commits the branch path and keeps the popup open                                                                                                                                                                        | `changeOnSelect commits branch paths while keeping the popup open`                            |
| K7  | multiple              | click leaves                                                                                | toggles leaf paths, stays open, marks columns `aria-multiselectable`, submits one hidden input per path                                                                                                                 | `multiple mode toggles leaf paths and stays open`                                             |
| K8  | `expandTrigger=hover` | pointer move over a branch                                                                  | highlights and expands it                                                                                                                                                                                               | `hover expandTrigger expands branches on pointer movement`                                    |
| L1  | `loadChildren`        | expand / expand a sibling / resolve                                                         | loads once, marks the option `data-loading` and its column `aria-busy`, aborts a superseded sibling load, descends after a keyboard-triggered load                                                                      | `loadChildren loads lazily, marks loading, descends by keyboard, and aborts superseded loads` |
| L2  | `loadChildren`        | reject / unmount                                                                            | emits `load-error(node, error)`; unmount aborts pending loads                                                                                                                                                           | `unmount aborts pending loads and rejected loads emit load-error`                             |
| S1  | `search`              | non-empty query                                                                             | publishes matching (loaded) leaf paths as `searchResults`; `selectPath` chooses one                                                                                                                                     | `search publishes matching paths that can be selected`                                        |
| C1  | controlled / disabled | select / keys                                                                               | controlled values emit and wait for the parent; disabled cascaders never open                                                                                                                                           | `controlled values wait for the parent; disabled cascaders never open`                        |
| C2  | no provider           | mount a part                                                                                | throws `VIZE_UI_CONTEXT_MISSING`                                                                                                                                                                                        | `parts require a Cascader provider`                                                           |
| U1  | pure helpers          | `findCascaderPath` / `flattenCascaderPaths` / `searchCascaderPaths` / `toCascaderSelection` | resolve, flatten (optionally including branches), search accent-insensitively, and normalize models                                                                                                                     | `pure helpers resolve, flatten, search, and normalize paths`                                  |

## SSR

Ids derive from the deterministic-id primitive, the popup renders in place
until the portal hydrates, and lazy loads only start after mount. Closed and
open trees render byte-identical markup and hydrate without warnings
(`renders byte-identical closed Cascader markup with the selected path label`,
`renders byte-identical open Cascader markup with columns for the selected
path`, `hydrates closed and open Cascaders without mismatches`).
