# Dismissable Layer Behavior

The dismissable layer primitive is a document-scoped overlay foundation. It does not render a component shell or visual styling; consumers keep ownership of source, markup, Teleport placement, inerting, focus guards, and scroll locks while sharing a deterministic dismissal stack.

| Scenario            | Required behavior                                                                                                                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Activation          | `activate()` joins the root element's `Document` stack when `root` resolves to a connected `Element`; `deactivate()` and `dispose()` release ownership exactly once.                                         |
| Top-layer routing   | Only the last activated, connected, enabled layer in a document is topmost and eligible for outside pointer, outside focus, and Escape dismissal.                                                            |
| Branches            | Reactive `branches` and imperative `registerBranch()` roots are treated as inside the layer, including portalled content and composed Shadow DOM paths.                                                      |
| Outside pointer     | A pointer down, mouse down fallback, or touch start outside the top layer emits immutable `pointer-down-outside` evidence, then `onInteractOutside`, then `onDismiss` unless prevented or a callback throws. |
| Outside focus       | A `focusin` outside the top layer emits immutable `focus-outside` evidence with `relatedTarget`, then `onInteractOutside`, then `onDismiss` unless prevented or a callback throws.                           |
| Escape routing      | An unprevented, non-composing `Escape` keydown in the document emits immutable `escape-key` evidence and then `onDismiss` unless prevented or a callback throws.                                             |
| Prevention          | Calling `preventDefault()` on pointer, focus, or Escape evidence prevents `onDismiss` for that native event without mutating the original event.                                                             |
| Reactive enablement | `enabled`, `outsidePointerDown`, `outsideFocus`, and `escapeKey` are re-read for every event and during synchronous option refreshes.                                                                        |
| Root migration      | Moving a root within the same document preserves stack order; moving across documents releases old listeners before attaching to the new document.                                                           |
| Disconnected roots  | Disconnecting a top layer removes its topmost eligibility on the next mutation turn and restores the next eligible parent layer.                                                                             |
| SSR and hydration   | `useDismissableLayer()` renders deterministic inactive props on the server, touches no global document during render, and activates after client mount without replacing hydrated nodes.                     |
| Runtime diagnostics | Invalid roots, branches, booleans, callbacks, out-of-scope usage, and post-disposal calls throw stable `VIZE_UI_DISMISSABLE_LAYER_*` diagnostics.                                                            |
