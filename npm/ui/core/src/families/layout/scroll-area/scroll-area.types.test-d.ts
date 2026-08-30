/** Compile-only assertions for the public ScrollArea contract. */

import { ScrollArea } from "./scroll-area.ts";
import type { ComponentPublicInstance } from "vue";
import { defineComponent } from "vue";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";
import type {
  ScrollAreaAriaState,
  ScrollAreaAs,
  ScrollAreaDirection,
  ScrollAreaEmits,
  ScrollAreaExpose,
  ScrollAreaLength,
  ScrollAreaOrientation,
  ScrollAreaOverflow,
  ScrollAreaOverscrollBehavior,
  ScrollAreaProps,
  ScrollAreaResolvedLayout,
  ScrollAreaResolvedLength,
  ScrollAreaRootElement,
  ScrollAreaScrollBehavior,
  ScrollAreaScrollbarGutter,
  ScrollAreaScrollbarWidth,
  ScrollAreaSlotState,
  ScrollAreaState,
  ScrollAreaStyle,
} from "./scroll-area.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: ScrollAreaExpose;

type _AsIsPolymorphic = Expect<Equal<ScrollAreaAs, PrimitiveAs>>;
type _RootElementIsPrimitive = Expect<Equal<ScrollAreaRootElement, PrimitiveElement>>;
type _OrientationIsClosed = Expect<
  Equal<ScrollAreaOrientation, "vertical" | "horizontal" | "both">
>;
type _DirectionIsClosed = Expect<Equal<ScrollAreaDirection, "ltr" | "rtl">>;
type _ScrollBehaviorIsClosed = Expect<Equal<ScrollAreaScrollBehavior, "auto" | "smooth">>;
type _OverscrollBehaviorIsClosed = Expect<
  Equal<ScrollAreaOverscrollBehavior, "auto" | "contain" | "none">
>;
type _ScrollbarGutterIsClosed = Expect<
  Equal<ScrollAreaScrollbarGutter, "auto" | "stable" | "stable both-edges">
>;
type _ScrollbarWidthIsClosed = Expect<Equal<ScrollAreaScrollbarWidth, "auto" | "thin" | "none">>;
type _LengthAcceptsCssOrNumber = Expect<Equal<ScrollAreaLength, string | number>>;
type _ResolvedLengthIsString = Expect<Equal<ScrollAreaResolvedLength, string>>;
type _OverflowIsNative = Expect<Equal<ScrollAreaOverflow, "auto" | "hidden">>;
type _StateIsStable = Expect<Equal<ScrollAreaState, "scrollable">>;
type _EmitsAreStrict = Expect<Equal<ScrollAreaEmits, { scroll: [nativeEvent: Event] }>>;
type _ExposeViewportIsNative = Expect<Equal<typeof exposed.viewport, HTMLDivElement | null>>;
type _ExposeRootIsPrimitive = Expect<Equal<typeof exposed.root, ScrollAreaRootElement | null>>;
type _ExposeOrientationIsLiteral = Expect<Equal<typeof exposed.orientation, ScrollAreaOrientation>>;
type _ExposeDirectionIsLiteral = Expect<Equal<typeof exposed.dir, ScrollAreaDirection>>;
type _ExposeStyleIsStrict = Expect<Equal<typeof exposed.style, ScrollAreaStyle>>;
type _AriaStateIsStrict = Expect<
  Equal<
    ScrollAreaAriaState,
    {
      readonly ariaLabel: string | undefined;
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
    }
  >
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof ScrollAreaProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "blockSize"
    | "dir"
    | "focusable"
    | "inlineSize"
    | "maxBlockSize"
    | "maxInlineSize"
    | "orientation"
    | "overscrollBehavior"
    | "scrollBehavior"
    | "scrollbarGutter"
    | "scrollbarWidth"
  >
>;
type _SlotStateIsStrict = Expect<
  Equal<
    ScrollAreaSlotState,
    {
      readonly ariaLabel: string | undefined;
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
      readonly as: ScrollAreaAs;
      readonly orientation: ScrollAreaOrientation;
      readonly dir: ScrollAreaDirection;
      readonly focusable: boolean;
      readonly blockSize: ScrollAreaResolvedLength;
      readonly inlineSize: ScrollAreaResolvedLength;
      readonly maxBlockSize: ScrollAreaResolvedLength;
      readonly maxInlineSize: ScrollAreaResolvedLength;
      readonly overflowX: ScrollAreaOverflow;
      readonly overflowY: ScrollAreaOverflow;
      readonly overscrollBehavior: ScrollAreaOverscrollBehavior;
      readonly scrollBehavior: ScrollAreaScrollBehavior;
      readonly scrollbarGutter: ScrollAreaScrollbarGutter;
      readonly scrollbarWidth: ScrollAreaScrollbarWidth;
      readonly labelled: boolean;
      readonly described: boolean;
      readonly state: ScrollAreaState;
      readonly style: ScrollAreaStyle;
    }
  >
>;
type _ResolvedLayoutOmitsAriaAndHost = Expect<
  Equal<
    keyof ScrollAreaResolvedLayout,
    | "blockSize"
    | "dir"
    | "focusable"
    | "inlineSize"
    | "maxBlockSize"
    | "maxInlineSize"
    | "orientation"
    | "overflowX"
    | "overflowY"
    | "overscrollBehavior"
    | "scrollBehavior"
    | "scrollbarGutter"
    | "scrollbarWidth"
    | "state"
    | "style"
  >
>;

const publicProps = {
  ariaDescribedby: "scroll-help",
  ariaLabelledby: "scroll-title",
  as: "section",
  blockSize: 240,
  dir: "rtl",
  focusable: true,
  inlineSize: "100%",
  maxBlockSize: "60vh",
  maxInlineSize: "40rem",
  orientation: "both",
  overscrollBehavior: "contain",
  scrollBehavior: "smooth",
  scrollbarGutter: "stable both-edges",
  scrollbarWidth: "thin",
} satisfies ScrollAreaProps;
const CustomRoot = defineComponent({
  name: "ScrollAreaTypeHost",
  setup() {
    return () => null;
  },
});
const componentProps: InstanceType<typeof ScrollArea>["$props"] = publicProps;
const componentHostProps = {
  as: CustomRoot,
  orientation: "horizontal",
} satisfies ScrollAreaProps;
const slotState: ScrollAreaSlotState = {
  ariaDescribedby: undefined,
  ariaLabel: "Activity",
  ariaLabelledby: undefined,
  as: "div",
  blockSize: "auto",
  described: false,
  dir: "ltr",
  focusable: false,
  inlineSize: "auto",
  labelled: true,
  maxBlockSize: "none",
  maxInlineSize: "none",
  orientation: "vertical",
  overflowX: "hidden",
  overflowY: "auto",
  overscrollBehavior: "auto",
  scrollBehavior: "auto",
  scrollbarGutter: "auto",
  scrollbarWidth: "auto",
  state: "scrollable",
  style: {
    "--vize-ui-scroll-area-block-size": "auto",
    "--vize-ui-scroll-area-inline-size": "auto",
    "--vize-ui-scroll-area-max-block-size": "none",
    "--vize-ui-scroll-area-max-inline-size": "none",
    "--vize-ui-scroll-area-overscroll-behavior": "auto",
    "--vize-ui-scroll-area-overflow-x": "hidden",
    "--vize-ui-scroll-area-overflow-y": "auto",
    "--vize-ui-scroll-area-scroll-behavior": "auto",
    "--vize-ui-scroll-area-scrollbar-gutter": "auto",
    "--vize-ui-scroll-area-scrollbar-width": "auto",
  },
};
const primitiveRoot: ScrollAreaRootElement = {} as ComponentPublicInstance;

// @ts-expect-error orientation is intentionally limited to native scroll axes.
const invalidOrientation: ScrollAreaOrientation = "diagonal";

// @ts-expect-error writing direction is explicit LTR or RTL.
const invalidDirection: ScrollAreaDirection = "auto";

// @ts-expect-error scroll behavior is limited to native CSS keywords.
const invalidScrollBehavior: ScrollAreaScrollBehavior = "instant";

// @ts-expect-error overscroll behavior is limited to native CSS keywords.
const invalidOverscroll: ScrollAreaOverscrollBehavior = "bounce";

// @ts-expect-error scrollbar gutter accepts only stable native policies.
const invalidGutter: ScrollAreaScrollbarGutter = "both-edges";

// @ts-expect-error focusable stays boolean rather than stringly typed.
const badFocusable = { focusable: "true" } satisfies ScrollAreaProps;

void ScrollArea;
void badFocusable;
void componentHostProps;
void componentProps;
void exposed;
void invalidDirection;
void invalidGutter;
void invalidOrientation;
void invalidOverscroll;
void invalidScrollBehavior;
void primitiveRoot;
void publicProps;
void slotState;
