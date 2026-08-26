# History behavior contract

Normative state × input → outcome table for `@vizejs/ui/history`. Every row is
exercised by `src/history*.test.ts`; compile-only assertions live in
`src/history.types.test-d.ts`.

| #   | State                     | Input                                           | Outcome                                                                  | Proven by             |
| --- | ------------------------- | ----------------------------------------------- | ------------------------------------------------------------------------ | --------------------- |
| H1  | applied change            | `push` then `undo` and `redo`                   | entries replay in reverse and forward order with reactive depths         | timeline test         |
| H2  | undone steps pending redo | new `push`                                      | the redo timeline clears                                                 | timeline test         |
| H3  | same coalescing key       | pushes within the coalescing window             | steps merge keeping the first undo and the latest redo                   | coalescing test       |
| H4  | same coalescing key       | pushes beyond the window or after undo          | steps stay separate                                                      | coalescing test       |
| H5  | snapshot push             | equal before and after values                   | no-op snapshots are dropped                                              | snapshot test         |
| H6  | open transaction          | staged pushes then `commit`                     | everything folds into one labeled step that undoes and redoes atomically | transaction test      |
| H7  | open transaction          | `rollback` or a throwing `transaction` callback | staged entries undo in reverse and the failure surfaces                  | transaction test      |
| H8  | nested transactions       | inner settle then outer settle                  | inner frames fold into the outer step; LIFO misuse throws                | transaction test      |
| H9  | open transaction          | `undo`, `redo`, or `clear`                      | stable transaction diagnostic rejects the call                           | transaction test      |
| H10 | restoring state           | `push` from a reactive mirror                   | the push is discarded so replay cannot corrupt the timeline              | restoring test        |
| H11 | timeline at `limit`       | new step                                        | the oldest step drops                                                    | limit test            |
| H12 | throwing entry            | `undo` or `redo`                                | the failure surfaces, the entry drops, and the timeline stays usable     | failure test          |
| H13 | editable session          | `begin`/`update` then `commit`                  | live writes bypass history and one snapshot step covers the session      | editable test         |
| H14 | editable session          | `cancel` or an unchanged `commit`               | the pre-edit value restores, or no step is pushed                        | editable test         |
| H15 | invalid options           | malformed entries, snapshots, or options        | stable runtime diagnostics reject the misuse                             | diagnostics test      |
| H16 | active controller         | dispose or Vue scope stop                       | both timelines clear and imperative calls become terminal                | lifecycle test        |
| H17 | concurrent SSR requests   | identical consumers                             | byte-identical markup with no request-global timeline state              | SSR test              |
| H18 | SSR followed by hydration | undoable interaction                            | host identity remains and undo/redo state renders without warnings       | hydration test        |
| H19 | DOM, SSR, and Vapor lanes | authored consumer compiles                      | every renderer accepts the same controller surface                       | renderer gate         |
| H20 | root and subpath consumer | only history is retained                        | equal CSS-free bundles exclude unrelated component families              | tree-shaking gate     |
| H21 | public TypeScript API     | mutation or invalid options                     | compile-only assertions reject misuse                                    | type declaration test |

## Accessibility obligation

Undo is an error-recovery affordance: keep Undo and Redo reachable through
visible, focusable controls with `disabled` state bound to `canUndo` and
`canRedo`, not only through keyboard accelerators. Use `undoLabel` and
`redoLabel` to give those controls specific accessible names (for example
"Undo Typing"), and announce destructive coalesced steps before dropping
them. Field editing should flow through the editable transaction so one
keystroke-level mistake never needs dozens of undo activations.
