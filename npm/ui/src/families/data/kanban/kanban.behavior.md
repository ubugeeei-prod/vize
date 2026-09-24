# Kanban behavior contract

`@vizejs/ui/kanban` renders a board of labelled columns (`kanban-column.vue`) holding focusable cards
(`kanban-card.vue`); `kanban.vue` is the generic root (`Card`, `ColumnId`). Pointer and keyboard moves use
the shared `drag-and-drop` foundation (cards are sources and "before" targets; column lists are "end"
targets). Every row is proven by the named test in `kanban.test.ts` or `kanban-ssr.test.ts`.

| #   | State             | Input                            | Outcome                                                                                                           | Proven by                                                                       |
| --- | ----------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| K1  | any               | render                           | labelled board, `role="group"` columns named by their header, lists of `listitem` cards described by instructions | `renders labelled column groups with lists of focusable cards and instructions` |
| K2  | card focus        | arrows, Home/End                 | roving focus moves within a column and across columns (index clamped, empty columns skipped)                      | `arrow keys move the roving focus within and across columns`                    |
| K3  | card focus        | Space, arrows, Enter             | keyboard drag cycles valid targets, drops into another column, emits `move` + `update:modelValue`, keeps focus    | `keyboard drag moves a card into another column and keeps focus on it`          |
| K4  | keyboard drag     | Escape                           | cancels without moving                                                                                            | `Escape cancels a keyboard drag without moving`                                 |
| K5  | limits and vetoes | keyboard drag                    | full (`limit`) and disabled columns and `canMove` vetoes are never offered as targets                             | `WIP limits, disabled columns, and canMove veto drop targets`                   |
| K6  | pointer drag      | release over a card's lower half | the drop edge is published and the card lands after it                                                            | `dropping on a card's bottom edge inserts after it`                             |
| K7  | custom boards     | import                           | `KanbanColumn` and `KanbanCard` parts are exported                                                                | `column and card parts are exported for custom boards`                          |
| K8  | SSR               | two requests + hydrate           | byte-identical markup with one tab stop and no drag state; warning-free hydration                                 | `renders identical board markup across SSR requests and hydrates cleanly`       |
