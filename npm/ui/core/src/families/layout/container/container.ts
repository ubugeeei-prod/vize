export type {
  ContainerElement,
  ContainerExpose,
  ContainerLength,
  ContainerResolvedLayout,
  ContainerResolvedLength,
  ContainerSize,
  ContainerSlotState,
  ContainerStyle,
} from "./container-types.ts";

/** Headless, CSS-first max-width container powered by native logical properties. */
export { default as Container } from "./container.vue";
