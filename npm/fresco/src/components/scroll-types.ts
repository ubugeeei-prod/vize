import type {
  VirtualScrollAlignment,
  VirtualViewportRange,
} from "../composables/useVirtualViewport.js";

/** Slot state exposed for one retained ScrollBox item. */
export interface ScrollBoxItemSlot extends VirtualViewportRange {
  index: number;
  visible: boolean;
}

/** Public ScrollBox props for a bounded vertical viewport. */
export interface ScrollBoxProps {
  itemCount: number;
  viewportSize: number;
  modelValue?: number;
  overscan?: number;
  focused?: boolean;
  disabled?: boolean;
  wheelStep?: number;
  showScrollbar?: boolean;
  followOutput?: boolean;
  scrollbarTrack?: string;
  scrollbarThumb?: string;
}

/** Imperative methods exposed through a ScrollBox template ref. */
export interface ScrollBoxExposed {
  scrollTo(offset: number): boolean;
  scrollBy(delta: number): boolean;
  scrollIntoView(index: number, alignment?: VirtualScrollAlignment): boolean;
  range(): VirtualViewportRange;
}

/** ScrollBar layout axis. */
export type ScrollBarOrientation = "vertical" | "horizontal";

/** Public ScrollBar props for a bounded viewport indicator. */
export interface ScrollBarProps {
  itemCount: number;
  viewportSize: number;
  offset: number;
  length?: number;
  orientation?: ScrollBarOrientation;
  track?: string;
  thumb?: string;
  fg?: string;
  thumbFg?: string;
}
