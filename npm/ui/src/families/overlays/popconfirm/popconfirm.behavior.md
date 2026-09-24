# Popconfirm behavior contract

Normative behavior for the `@vizejs/ui/popconfirm` compound primitive: an inline,
anchored confirmation built on the Popover family (root, trigger, positioned content,
dismissable layer, focus scope). Every row is proven by the named test.

| State x input                                  | Observable outcome                                                                                                                             | Proven by                                                                    |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| closed, trigger click                          | The popover opens as `role="alertdialog"` labelled by the title and described by the description; Cancel receives focus.                       | `opens an alertdialog labelled by title and description with cancel focused` |
| `initialFocus="confirm"`                       | The Confirm action receives initial focus (`"none"` leaves focus to the popover).                                                              | `initialFocus can target confirm or leave focus to the popover`              |
| open, Confirm with a synchronous handler       | The handler runs once, the popover closes, and `confirmed` plus `update:open` fire.                                                            | `synchronous confirm closes and emits confirmed`                             |
| open, Confirm with a promise                   | State becomes `pending`: both actions are disabled, Confirm and content report `aria-busy`, Escape cannot cancel; resolve closes and confirms. | `async confirm stays open and busy until the promise resolves`               |
| pending promise rejects, or the handler throws | The popover stays open, actions re-enable, `error` fires with the reason, and content publishes `data-error`.                                  | `rejected confirm stays open, exposes the error, and emits error`            |
| open, Cancel / Escape / outside pointer-down   | The popover closes without confirming and `cancel` reports `cancel-button` or `dismiss`.                                                       | `cancel button, Escape, and outside pointer-down cancel with a reason`       |
| controlled `open`; root expose                 | Requests emit `update:open` until the parent accepts; `setOpen`, `confirm`, `cancel`, ids, `state`, and `error` are exposed.                   | `controlled open and root expose drive the confirmation`                     |
| `disabled` root                                | The trigger cannot open the confirmation.                                                                                                      | `disabled roots keep the trigger inert`                                      |
| parts outside the root                         | Mounting throws `VIZE_UI_CONTEXT_MISSING`.                                                                                                     | `parts require a Popconfirm root`                                            |
| SSR                                            | Closed markup is byte-identical across requests and contains no content.                                                                       | `renders byte-identical closed popconfirm markup across isolated requests`   |
| hydration with `defaultOpen`                   | Server markup already renders `role="alertdialog"` through `PopoverContent`'s `role` prop; hydration emits no warnings.                        | `renders default-open alertdialog markup and hydrates without diagnostics`   |
| public types                                   | Handlers resolve to nothing; state, focus target, and cancel reason are closed unions.                                                         | `src/families/overlays/popconfirm/popconfirm.types.test-d.ts`                |

## Components

| Component                | State x input                  | Outcome                                                                                              |
| ------------------------ | ------------------------------ | ---------------------------------------------------------------------------------------------------- |
| `popconfirm-root.vue`    | open requests, confirm handler | Wraps `PopoverRoot`, owns `idle`/`pending` state and the last error, refuses to close while pending. |
| `popconfirm-trigger.vue` | click                          | Renders `PopoverTrigger` with `data-vize-ui="popconfirm-trigger"`.                                   |
| `popconfirm-content.vue` | open, pending                  | Renders `PopoverContent` with title/description ids, initial focus, and `alertdialog` role.          |
| `popconfirm-confirm.vue` | click                          | Runs the handler; disabled and `aria-busy` while pending.                                            |
| `popconfirm-cancel.vue`  | click                          | Cancels with reason `cancel-button`; disabled while pending.                                         |

## Parts And Data

| Target  | Public contract                                                                                                             |
| ------- | --------------------------------------------------------------------------------------------------------------------------- |
| Root    | `data-vize-ui="popconfirm-root"`, `part="root"`, `data-popconfirm-state`, plus Popover `data-state`                         |
| Trigger | `data-vize-ui="popconfirm-trigger"`, `part="trigger"`, `data-popconfirm-state`                                              |
| Content | `data-vize-ui="popconfirm-content"`, `part="content"`, `data-state`, `data-error`, `aria-busy`; `title`/`description` parts |
| Actions | `data-vize-ui="popconfirm-confirm"` / `"popconfirm-cancel"`, `part`, `data-state`                                           |

Popconfirm ships no stylesheet.
