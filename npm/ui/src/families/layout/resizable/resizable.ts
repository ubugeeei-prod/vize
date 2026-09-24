/** Accessible, unstyled single-element resize wrapper with edge and corner handles. */
export { default as Resizable, default as ResizableRoot } from "./resizable-root.vue";
export { default as ResizableHandle } from "./resizable-handle.vue";
export {
  constrainResizableSize,
  resizeByDelta,
  resolveResizableEdge,
} from "./resizable-geometry.ts";
export type { ResizableConstraints } from "./resizable-geometry.ts";
export type {
  ResizableDirection,
  ResizableEdge,
  ResizableHandleExpose,
  ResizablePhysicalEdge,
  ResizableResizeEvent,
  ResizableRootExpose,
  ResizableSize,
  ResizableSlotState,
  ResizableSource,
  ResizableState,
} from "./resizable-types.ts";
