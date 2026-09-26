# P3-6 - Static events in once subtrees (2026-09-26)

The checked native plain-element once subtree now admits static-name DOM
listeners beside its mount-only props and text. Handler closures still read
the current render context; once freezes DOM updates, not handler reads.
Computed names, components, control flow, and other directives remain explicit
legacy selections until their independent once contracts are proved.

## Evidence

- Three sources under both prefix settings assert admission, exact generated
  native code snapshots and byte-equal retained templates. The root-button
  case also has byte-equal generated code. Nested nodes can have different
  local IDs between the native and retained allocators; snapshots retain the
  native IDs rather than disguising that difference.
- A checked-payload mutation changes the handler while emission receives a
  decoy source. Two code/map snapshots compare the full code and every decoded
  source-map segment with the retained lane for references and callbacks.
- TS-33 compares the pinned official compiler/runtime and the native lane.
  DOM props and text stay at their initial value while inline callbacks
  observe later values; click and Enter modifiers preserve their guards.
  The same contract holds under an inherited once parent. A `.once` listener
  fires once and is never reinstalled by later state changes. Identities remain
  stable until unmount removes the subtree.
- Three new sources under both prefix settings keep zero legacy walks and
  expression reparses. Existing allocation ceilings are unchanged.

P3-6's full semantic and performance exits remain open.
