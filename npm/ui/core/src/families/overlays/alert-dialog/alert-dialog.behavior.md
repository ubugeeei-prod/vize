# AlertDialog behavior contract

Normative state x input -> outcome table for `alert-dialog-content.vue`
(`@vizejs/ui/alert-dialog`). Every row is proven by the named mounted-DOM, SSR,
or compile-time test.

| Surface   | Contract                                                                                                                             |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Root      | `AlertDialogRoot` reuses Dialog controlled/uncontrolled state, deterministic ids, modal defaults, and slot/expose state.             |
| Content   | `AlertDialogContent` renders Dialog content with fixed `role="alertdialog"` and publishes an `alert-dialog-content` styling wrapper. |
| Dismissal | Outside pointer and focus dismissal are disabled by default; Escape and explicit action/cancel buttons may request closing.          |
| Labelling | Title and description aliases keep Dialog deterministic id wiring for `aria-labelledby` and `aria-describedby`.                      |
| Styling   | No component CSS is emitted beyond a scoped empty block; `data-vize-ui`, `part`, and `data-state` are the contract.                  |
| Packaging | Root and subpath consumers retain only AlertDialog plus required Dialog overlay utilities with zero CSS.                             |

| State x input                    | Outcome                                                                                              | Proven by                                                                                           |
| -------------------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| trigger click                    | Opens modal content with native `role="alertdialog"` and title/description id wiring.                | `opens as a labelled modal alertdialog with explicit close actions`                                 |
| outside pointer-down by default  | Leaves the alert dialog open so destructive confirmations are not dismissed by backdrop accidents.   | `opens as a labelled modal alertdialog with explicit close actions`                                 |
| cancel or action click           | Uses Dialog close semantics and restores focus to the trigger.                                       | `opens as a labelled modal alertdialog with explicit close actions`                                 |
| opted-in outside pointer-down    | Emits the preventable outside event and closes when the event is not canceled.                       | `can opt into outside pointer dismissal`                                                            |
| SSR/hydration                    | Server markup is deterministic and contains no document-controller listeners or scroll-lock effects. | `renders deterministic alertdialog markup on the server`                                            |
| public types and consumer bundle | TypeScript rejects a custom role, while root and subpath imports emit equivalent zero-CSS bundles.   | `src/families/overlays/alert-dialog/alert-dialog.types.test-d.ts`, `scripts/check-tree-shaking.mjs` |
