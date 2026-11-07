/** Compile-time contracts for public Fresco composables. */

import { createVirtualViewport } from "./useVirtualViewport.js";

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
