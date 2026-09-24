/** Compile-only assertions for the public Tour contract. */

import { h } from "vue";

import {
  Tour,
  TourArrow,
  TourClose,
  TourContent,
  TourDescription,
  TourNext,
  TourPrev,
  TourProgress,
  TourRoot,
  TourSpotlight,
  TourStep,
  TourTitle,
  padTourRect,
  resolveTourTarget,
  type TourCloseReason,
  type TourContentExpose,
  type TourContentPlacement,
  type TourContentSlotState,
  type TourControlSlotState,
  type TourDismissReason,
  type TourMissingTargetBehavior,
  type TourProgressSlotState,
  type TourRootExpose,
  type TourSlotState,
  type TourSpotlightExpose,
  type TourSpotlightRect,
  type TourState,
  type TourStepDefinition,
  type TourStepValue,
  type TourTarget,
  type TourTargetState,
} from "./tour.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface ProductStep extends TourStepDefinition {
  readonly value: "welcome" | "search" | "profile";
  readonly title: string;
}

declare const root: TourRootExpose<ProductStep>;
declare const slot: TourSlotState<ProductStep>;
declare const contentSlot: TourContentSlotState;
declare const content: TourContentExpose;
declare const spotlight: TourSpotlightExpose;
declare const progress: TourProgressSlotState;
declare const control: TourControlSlotState;

type _StateIsLiteral = Expect<Equal<TourState, "closed" | "open">>;
type _TargetStateIsLiteral = Expect<
  Equal<TourTargetState, "missing" | "none" | "pending" | "resolved">
>;
type _MissingIsLiteral = Expect<Equal<TourMissingTargetBehavior, "center" | "skip">>;
type _CloseReasonIsLiteral = Expect<Equal<TourCloseReason, "close" | "skip">>;
type _DismissReasonIsLiteral = Expect<
  Equal<
    TourDismissReason,
    "close" | "escape-key" | "focus-outside" | "pointer-down-outside" | "skip"
  >
>;
type _StepValueIsInferred = Expect<
  Equal<TourStepValue<ProductStep>, "welcome" | "search" | "profile">
>;
type _DefaultStepValueIsString = Expect<Equal<TourStepValue, string>>;
type _SlotStepKeepsConsumerFields = Expect<Equal<typeof slot.step, ProductStep | null>>;
type _SlotValueIsNarrow = Expect<Equal<typeof slot.value, "welcome" | "search" | "profile" | null>>;
type _RootGoToIsNarrow = Expect<
  Equal<Parameters<typeof root.goTo>[0], "welcome" | "search" | "profile">
>;
type _RootTargetIsElement = Expect<Equal<typeof root.target, Element | null>>;
type _ContentPlacementIncludesCenter = Expect<
  Equal<Extract<TourContentPlacement, "center">, "center">
>;
type _ContentSlotPlacement = Expect<Equal<typeof contentSlot.placement, TourContentPlacement>>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SpotlightRect = Expect<Equal<typeof spotlight.rect, TourSpotlightRect | null>>;
type _ProgressFraction = Expect<Equal<typeof progress.fraction, number>>;
type _ControlLast = Expect<Equal<typeof control.last, boolean>>;
type _AliasIsRoot = Expect<Equal<typeof Tour, typeof TourRoot>>;
type _PadReturnsRect = Expect<Equal<ReturnType<typeof padTourRect>, TourSpotlightRect>>;
type _ResolveReturnsElement = Expect<Equal<ReturnType<typeof resolveTourTarget>, Element | null>>;

const selectorTarget: TourTarget = "#search";
const getterTarget: TourTarget = () => null;
void selectorTarget;
void getterTarget;

// @ts-expect-error numbers are not tour targets
const numericTarget: TourTarget = 1;
void numericTarget;

// @ts-expect-error unknown steps cannot be requested
root.goTo("billing");

// @ts-expect-error missing-target behavior is a closed union
const hidden: TourMissingTargetBehavior = "hide";
void hidden;

h(TourContent, { portalDisabled: true, closeOnEscape: false, ariaLabelledby: null });
h(TourClose, { reason: "skip" });
// @ts-expect-error close reasons are a closed union
h(TourClose, { reason: "later" });
h(TourSpotlight, { padding: 4, radius: 8, interactive: true });
h(TourStep, { value: "search" });
h(TourTitle, { as: "h3" });
// @ts-expect-error titles render heading-like elements only
h(TourTitle, { as: "section" });
h(TourDescription);
h(TourProgress);
h(TourPrev);
h(TourNext);
h(TourArrow);

declare const productSteps: readonly ProductStep[];
h(TourRoot, { steps: productSteps, defaultStep: "search" });
// Generic SFCs are callable component types: calling one infers `TStep` from `steps`.
declare const typedRoot: typeof TourRoot;
typedRoot({ steps: productSteps, defaultStep: "search", step: null });
// @ts-expect-error step values are inferred from the steps prop
typedRoot({ steps: productSteps, defaultStep: "billing" });
typedRoot({
  steps: productSteps,
  "onUpdate:step": (value) => {
    type _EmittedStepIsNarrow = Expect<Equal<typeof value, "welcome" | "search" | "profile">>;
  },
});
// @ts-expect-error steps are required
h(TourRoot, { defaultOpen: true });
