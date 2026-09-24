/** Compile-only assertions for the public Carousel contract. */

import {
  Carousel,
  CarouselAutoplayToggle,
  CarouselIndicator,
  CarouselIndicatorGroup,
  CarouselNext,
  CarouselPrevious,
  CarouselRoot,
  CarouselSlide,
  CarouselViewport,
  nearestSlideIndex,
  resolveSlideIndex,
  type CarouselAutoplayState,
  type CarouselButtonExpose,
  type CarouselChangeReason,
  type CarouselDirection,
  type CarouselFocusBehavior,
  type CarouselIndicatorGroupExpose,
  type CarouselIndicatorSlotState,
  type CarouselOrientation,
  type CarouselPauseReason,
  type CarouselRootExpose,
  type CarouselSlideExpose,
  type CarouselSlideSlotState,
  type CarouselSlideState,
  type CarouselSlotState,
  type CarouselViewportExpose,
} from "./carousel.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: CarouselRootExpose;
declare const viewport: CarouselViewportExpose;
declare const slide: CarouselSlideExpose;
declare const button: CarouselButtonExpose;
declare const group: CarouselIndicatorGroupExpose;

type _OrientationIsLiteral = Expect<Equal<CarouselOrientation, "horizontal" | "vertical">>;
type _DirectionIsLiteral = Expect<Equal<CarouselDirection, "ltr" | "rtl">>;
type _AutoplayIsLiteral = Expect<Equal<CarouselAutoplayState, "paused" | "playing" | "stopped">>;
type _FocusBehaviorIsLiteral = Expect<Equal<CarouselFocusBehavior, "none" | "pause" | "stop">>;
type _SlideStateIsLiteral = Expect<Equal<CarouselSlideState, "active" | "inactive">>;
type _PauseReasonIsLiteral = Expect<
  Equal<CarouselPauseReason, "dragging" | "focus" | "hidden" | "hover" | "reduced-motion">
>;
type _ReasonIsLiteral = Expect<
  Equal<
    CarouselChangeReason,
    "api" | "autoplay" | "drag" | "indicator" | "keyboard" | "next" | "previous" | "scroll"
  >
>;
type _SlotStateIsExact = Expect<
  Equal<
    CarouselSlotState,
    {
      readonly index: number;
      readonly slideCount: number;
      readonly canScrollPrev: boolean;
      readonly canScrollNext: boolean;
      readonly autoplay: CarouselAutoplayState;
      readonly dragging: boolean;
      readonly orientation: CarouselOrientation;
    }
  >
>;
type _SlideSlotIsExact = Expect<
  Equal<
    CarouselSlideSlotState,
    {
      readonly index: number;
      readonly active: boolean;
      readonly inView: boolean | null;
      readonly state: CarouselSlideState;
    }
  >
>;
type _IndicatorSlotIsExact = Expect<
  Equal<CarouselIndicatorSlotState, { readonly index: number; readonly active: boolean }>
>;
type _RootElement = Expect<Equal<typeof root.element, HTMLElement | null>>;
type _ScrollToReports = Expect<Equal<typeof root.scrollTo, (index: number) => boolean>>;
type _ViewportElement = Expect<Equal<typeof viewport.element, HTMLDivElement | null>>;
type _SlideElement = Expect<Equal<typeof slide.element, HTMLDivElement | null>>;
type _ButtonElement = Expect<Equal<typeof button.element, HTMLButtonElement | null>>;
type _GroupElement = Expect<Equal<typeof group.element, HTMLDivElement | null>>;
type _AliasIsRoot = Expect<Equal<typeof Carousel, typeof CarouselRoot>>;
type _ResolveReturnsIndex = Expect<Equal<ReturnType<typeof resolveSlideIndex>, number>>;
type _NearestReturnsIndex = Expect<Equal<ReturnType<typeof nearestSlideIndex>, number>>;

void CarouselAutoplayToggle;
void CarouselIndicator;
void CarouselIndicatorGroup;
void CarouselNext;
void CarouselPrevious;
void CarouselSlide;
void CarouselViewport;

// @ts-expect-error autoplay states are a closed union.
const _unknownAutoplay: CarouselAutoplayState = "running";
// @ts-expect-error change reasons are a closed union.
const _unknownReason: CarouselChangeReason = "swipe";
// @ts-expect-error exposed state is read-only.
root.index = 2;
