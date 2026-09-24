# Confirm behavior contract

Normative state x input -> outcome table for `@vizejs/ui/confirm` (`confirm-provider.vue` plus
`useConfirm()` / `createConfirmQueue()` in `confirm-runtime.ts`). The provider renders the
existing AlertDialog primitive (`alert-dialog-content.vue` inside the Dialog root, portal,
overlay, title, description, and close parts) for the request at the head of a FIFO queue.
Every row names the test that proves it (`confirm.test.ts`, `confirm-ssr.test.ts`, or
`confirm.types.test-d.ts`).

| Surface | Contract                                                                                                                                            |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| API     | `useConfirm<Data>()` returns `confirm(options): Promise<boolean>`, `choose(options): Promise<Value \| null>`, `pending`, and `cancelAll()`.         |
| Queue   | Requests are shown one at a time in FIFO order; each settles exactly once.                                                                          |
| Dialog  | The active request renders `role="alertdialog"`, modal, title-labelled, focus-contained, with outside presses ignored.                              |
| Data    | `data-vize-ui="confirm-provider"`, `data-state` (`idle`/`pending`), `data-pending`, `data-confirm-kind`, `data-confirm-action`, `data-destructive`. |
| SSR     | The queue touches no DOM and starts no timers; server output is deterministic and hydrates without diagnostics.                                     |

| State x input                                        | Outcome                                                                                                                                               | Proven by                                                                               |
| ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| idle x `confirm()`                                   | Opens a labelled, described alertdialog, focuses the cancel action first, marks destructive actions.                                                  | `confirm() opens a labelled alertdialog and resolves true from the accepting action`    |
| pending x accepting action                           | Resolves `true`, emits `settle`, closes, and returns focus to the previously focused element.                                                         | `confirm() opens a labelled alertdialog and resolves true from the accepting action`    |
| pending x cancel action or Escape                    | Resolves `false`; provider labels apply; requests without a description reference no description.                                                     | `cancel and Escape resolve false, and missing descriptions omit aria-describedby`       |
| pending x further requests; `cancelAll()`            | Later requests wait in FIFO order and `pending` counts them; `cancelAll()` settles every request `false`.                                             | `requests queue in FIFO order and cancelAll settles every pending request`              |
| idle x `choose()`                                    | Renders one button per action; resolves the typed chosen value, or `null` on cancel.                                                                  | `choose() resolves the typed action value or null`                                      |
| empty title, duplicate or empty action values        | Rejects with `VIZE_UI_CONFIRM_OPTION` without queueing.                                                                                               | `invalid options throw stable diagnostics`                                              |
| `content` slot                                       | Replaces the body; `resolve(value)` settles with a known action value, `cancel()` settles cancelled, unknown values are ignored; `data` is forwarded. | `the content slot replaces the body and settles through resolve and cancel`             |
| provider unmount                                     | Pending requests resolve as not confirmed, and later requests resolve immediately.                                                                    | `unmounting the provider resolves pending and later requests as not confirmed`          |
| default portal                                       | The alert dialog teleports to `body` after mount and still settles requests.                                                                          | `the provider teleports the dialog to the body by default`                              |
| `useConfirm()` without provider                      | Throws `VIZE_UI_CONFIRM_PROVIDER_MISSING`.                                                                                                            | `useConfirm outside a provider throws a stable diagnostic`                              |
| server render                                        | Idle output is byte-identical with no dialog layer.                                                                                                   | `renders byte-identical idle markup with no dialog layer`                               |
| server render with a request made during child setup | Markup stays idle and deterministic; idle markup hydrates with zero diagnostics.                                                                      | `requests made while rendering on the server never change markup; idle markup hydrates` |
| public types                                         | `choose` infers the literal action union, `data` follows the `Data` argument, and invalid options fail.                                               | `src/families/overlays/confirm/confirm.types.test-d.ts`                                 |
