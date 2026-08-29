/** Compile-only assertions for the public Skeleton contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  SkeletonAriaState,
  SkeletonElement,
  SkeletonExpose,
  SkeletonSlotState,
  SkeletonState,
  SkeletonStyle,
} from "./skeleton.ts";
import { Skeleton } from "./skeleton.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: SkeletonExpose;
declare const componentTarget: Component;

type _ElementIsRenderable = Expect<Equal<SkeletonElement, Element | ComponentPublicInstance>>;
type _StateIsLiteral = Expect<Equal<SkeletonState, "hidden" | "loaded" | "loading">>;
type _AriaStateIsLiteral = Expect<Equal<SkeletonAriaState, "decorative" | "status">>;
type _ExposeLoadingIsBoolean = Expect<Equal<typeof exposed.loading, boolean>>;
type _ExposeVisibleIsBoolean = Expect<Equal<typeof exposed.visible, boolean>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, SkeletonState>>;
type _ExposeAriaStateIsLiteral = Expect<Equal<typeof exposed.ariaState, SkeletonAriaState>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    SkeletonSlotState,
    {
      readonly loading: boolean;
      readonly visible: boolean;
      readonly state: SkeletonState;
      readonly ariaState: SkeletonAriaState;
    }
  >
>;
type _StyleHooksAreLiteral = Expect<
  Equal<
    SkeletonStyle,
    {
      readonly "--vize-ui-skeleton-block-size": string;
      readonly "--vize-ui-skeleton-inline-size": string;
    }
  >
>;

const exposedElement: SkeletonElement | null = exposed.element;
const customHost: InstanceType<typeof Skeleton>["$props"] = {
  ariaHidden: false,
  ariaLabel: "Loading profile",
  as: componentTarget,
  blockSize: "2rem",
  inlineSize: "12rem",
  loading: true,
  visible: true,
};
const slotState: SkeletonSlotState = {
  ariaState: "status",
  loading: true,
  state: "loading",
  visible: true,
};
const style: SkeletonStyle = {
  "--vize-ui-skeleton-block-size": "1em",
  "--vize-ui-skeleton-inline-size": "100%",
};

// @ts-expect-error Skeleton has a closed state token contract.
const invalidState: SkeletonState = "pending";

// @ts-expect-error ARIA state is decorative or status only.
const invalidAriaState: SkeletonAriaState = "alert";

// @ts-expect-error component props require boolean loading.
const badLoading: InstanceType<typeof Skeleton>["$props"] = { loading: "true" };

// @ts-expect-error slot state exposes boolean visibility.
const badSlotState: SkeletonSlotState = { loading: true, visible: "true" };

void Skeleton;
void badLoading;
void badSlotState;
void customHost;
void exposedElement;
void invalidAriaState;
void invalidState;
void slotState;
void style;
