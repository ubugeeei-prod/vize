# Inert outside behavior contract

`createInertOutside` isolates the rendered sibling subtrees outside a modal root. It owns only
`inert` and `aria-hidden`; focus movement, roles, visual obscuring, dismissal, and scroll locking
remain separate primitives so applications can compose the correct modal behavior.

| Concern          | Contract                                                                                                                              |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Masking          | The smallest rendered sibling subtrees outside all allowed roots receive `inert`, `aria-hidden`, or both.                             |
| Restoration      | Pre-existing attribute values and boolean presence are restored exactly after disablement, deactivation, root migration, or disposal. |
| Nesting          | Earlier layers allow later portalled roots; the latest layer may isolate its parent until it closes.                                  |
| Branches         | Reactive branches preserve exceptional or portalled content without exposing unrelated siblings.                                      |
| Mutation         | One document observer watches the document plus reachable open shadow roots, then batches rendered-tree and owned-attribute repair.   |
| Shadow DOM       | Open shadow roots and assigned slots follow composed, rendered-tree paths; unrendered light children are ignored.                     |
| Reactivity       | Root, branch, mode, and enablement changes recompute synchronously without replacing the controller.                                  |
| Documents        | Root migration restores the old document before acquiring ownership in the new document.                                              |
| Server rendering | Setup performs no global DOM access; a nullable root joins the stack only after hydration mount.                                      |
| Vapor            | A public composable fixture must compile without diagnostics in native DOM, SSR, and Vapor lanes.                                     |
| Styling          | The primitive emits no CSS; callers must visually obscure content made inert, as required by the HTML Standard.                       |
| Tree shaking     | Root and subpath consumers emit identical JavaScript, retain no unrelated families, and emit zero CSS.                                |

`both` is the safe default: native `inert` blocks focus, selection, editing, and pointer targeting,
while `aria-hidden` keeps the isolated subtree out of accessibility APIs that do not yet model
`inert`. The `aria-hidden` mode is only appropriate when another primitive supplies equivalent
interaction blocking. Attribute ownership deliberately stays separate from backdrop styling.
