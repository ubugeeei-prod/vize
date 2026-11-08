/** Compile-time contracts for public Fresco composables. */

import { createFocusManager, type FocusRegistrationOptions } from "./useFocus.js";
import { createVirtualViewport } from "./useVirtualViewport.js";

const registration: FocusRegistrationOptions = {
  parentId: "dialog",
  delegateFocusTo: "field",
};
const focus = createFocusManager();
focus.register("panel", registration);
focus.setDelegate("panel", "field");
focus.trapFocus("panel");
focus.releaseFocusTrap("panel");

// @ts-expect-error - active trap state is read through manager methods.
focus.activeTrapId.value = "outside";

// @ts-expect-error - focus delegation uses stable string IDs.
focus.setDelegate("panel", 1);

const viewport = createVirtualViewport({ itemCount: 100, viewportSize: 10 });

viewport.scrollTo(20);
viewport.setItemCount(50);

// @ts-expect-error - public viewport state is mutated through invariant-preserving methods.
viewport.itemCount.value = 10;
// @ts-expect-error - public viewport state is mutated through invariant-preserving methods.
viewport.viewportSize.value = 5;
// @ts-expect-error - public viewport state is mutated through invariant-preserving methods.
viewport.overscan.value = 4;
// @ts-expect-error - public viewport state is mutated through invariant-preserving methods.
viewport.offset.value = 0;
