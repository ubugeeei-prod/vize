/** Existing public APIs exercised with real pointer input and document selection. */
export const pointerSelectionExamples = [
  {
    name: "pointer",
    title: "Inspect a guide pad's actual pointer events",
    purpose:
      "Read the latest accepted mouse pointer snapshot and distinguish target membership from retained coordinates.",
    observe:
      "Move into the pad, press and release, then leave. Client coordinates and pressure follow native mouse Pointer Events; Inside changes on leaving without clearing the snapshot. Hide the pad, move elsewhere, then show it again: the removed target stops receiving events and the replacement resumes observation.",
    context:
      "usePointer receives the reactive pad target and a fixed mouse-only snapshot filter. Its shallow readonly state records client coordinates in this iframe, pressure and the pointer identifier; it is not element-relative geometry. The default provider also accepts touch and pen snapshots; this example filters those kinds on pointerdown, pointermove and pointerup, and its native browser checks cover mouse input. Pointerleave clears Inside regardless of the snapshot filter. Hiding a target does not reset the latest snapshot. Target replacement removes old passive listeners; component scope disposal stops the target watcher. SSR exposes the initial zero snapshot, null pointer kind and false inside flag; no browser host is replaced.",
  },
  {
    name: "mouse-in-element",
    title: "Compare native client and guide-relative coordinates",
    purpose:
      "Read an element's current geometry on actual mouse moves and preserve its last inside location when the pointer leaves.",
    observe:
      "Move inside the pad, shift it and move again. Relative coordinates use the new transformed rectangle. Leaving with handleOutside: false updates Outside but freezes the last inside coordinates. Stop relative readings, then move: the underlying client coordinates still change while relative state stays fixed.",
    context:
      "useMouseInElement builds on the real client-coordinate mouse observer. Every relevant move reads getBoundingClientRect; transforms and scrolling are reflected on the next move, without an independent resize subscription. Its stop control freezes the relative watcher, not the underlying mouse subscription; the component scope disposes both. Zero-size rectangles count as outside. SSR has zero geometry and isOutside: true, with no measured browser position. The displayed coordinates describe this iframe, and the sample's native checkbox only changes its own CSS transform.",
  },
  {
    name: "draggable",
    title: "Move a local review note inside native bounds",
    purpose:
      "Use a separate pointer handle, reactive axis and disabled state while keeping a complete note inside an actual element rectangle.",
    observe:
      "Drag the handle toward the board edges: the whole note remains bounded, moves continue beyond the handle and Start/End counts describe actual drags. Select X only or lock pointer dragging. Edit the local note and move it with the keyboard buttons; the text stays exact. Stop listeners: pointer dragging stops while the explicit placement buttons still work.",
    context:
      "useDraggable exposes writable client-coordinate x/y and a style for fixed or absolute placement; this sample uses fixed positioning within its iframe. The real board ref supplies bounds, and the note's own dimensions are subtracted at the right/bottom edges. Axis and disabled affect pointer gestures, not arbitrary assignments to writable refs: the independent keyboard placement function explicitly clamps its writes. The chosen SSR position is a fallback; mounting measures the real board and places the same note inside it. A left mouse pointer starts the drag, and owner-document move/up/cancel listeners follow its pointer identifier. Stop is idempotent and scope disposal removes listeners; this example does not claim tested pen/touch gestures or persist the draft.",
  },
  {
    name: "text-selection",
    title: "Inspect native selection in an editable guide excerpt",
    purpose:
      "Keep selected text and native ranges synchronized when the browser mutates the same Selection object in place.",
    observe:
      "Focus the excerpt, select all, then use Shift and arrow keys for a shorter selection. The text and range shape refresh on actual selectionchange events without replacing the Selection object. Clear removes every range; a collapsed caret can have empty text and one range. Edit a new local passage and select it again.",
    context:
      "useTextSelection reads the real document after mounting and returns readonly selection-derived text, ranges and native range rectangles. Its revision invalidation handles in-place platform mutations. This is document Selection, including a native contenteditable region; it is not textarea/input selectionStart or selectionEnd. SSR and the first hydrating render expose empty text and no ranges, and scope disposal removes the listener. Clearing is an explicit native document operation. The editable excerpt owns local DOM text only: it sends no request and has no persistence or saved-state claim.",
  },
] as const;
