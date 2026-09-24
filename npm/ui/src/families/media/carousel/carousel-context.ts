import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CarouselScrollEdges } from "./carousel-geometry.ts";
import type {
  CarouselAutoplayState,
  CarouselChangeReason,
  CarouselDirection,
  CarouselOrientation,
  CarouselSlotState,
} from "./carousel-types.ts";

/** Imperative scroll surface registered by CarouselViewport. */
export interface CarouselViewportController {
  /** Align one slide with the viewport start edge. */
  readonly scrollToIndex: (index: number, smooth: boolean) => void;
}

/** Shared state and actions for the Carousel compound parts. */
export interface CarouselContextValue {
  readonly id: ComputedRef<string>;
  readonly viewportId: ComputedRef<string>;
  readonly getSlideId: (index: number) => string;
  readonly index: ComputedRef<number>;
  readonly slideCount: ComputedRef<number>;
  readonly orientation: ComputedRef<CarouselOrientation>;
  readonly dir: ComputedRef<CarouselDirection>;
  readonly loop: ComputedRef<boolean>;
  readonly draggable: ComputedRef<boolean>;
  readonly canScrollPrev: ComputedRef<boolean>;
  readonly canScrollNext: ComputedRef<boolean>;
  readonly autoplay: ComputedRef<CarouselAutoplayState>;
  readonly playing: ComputedRef<boolean>;
  readonly reducedMotion: ComputedRef<boolean>;
  readonly slotState: ComputedRef<CarouselSlotState>;
  readonly slideElements: Readonly<ShallowRef<ReadonlyMap<number, HTMLElement>>>;
  readonly isInView: (index: number) => boolean | null;
  readonly setInView: (index: number, inView: boolean) => void;
  readonly setEdges: (edges: CarouselScrollEdges) => void;
  readonly setDragging: (dragging: boolean) => void;
  readonly goTo: (index: number, reason: CarouselChangeReason) => boolean;
  readonly step: (delta: 1 | -1, reason: CarouselChangeReason) => boolean;
  readonly setPlaying: (playing: boolean) => boolean;
  readonly registerViewport: (controller: CarouselViewportController | null) => void;
  readonly registerSlide: (index: number, element: HTMLElement) => () => void;
  readonly registerIndicator: (index: number, element: HTMLButtonElement) => () => void;
  readonly focusIndicator: (index: number) => void;
}

export const carouselContext = createContext<CarouselContextValue>("Carousel");
