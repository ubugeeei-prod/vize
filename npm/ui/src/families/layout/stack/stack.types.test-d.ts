/** Compile-only assertions for the public stack contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  StackAlign,
  StackAxis,
  StackElement,
  StackExpose,
  StackFlexDirection,
  StackGap,
  StackJustify,
  StackResolvedLayout,
  StackSlotState,
  StackState,
  StackStyle,
} from "./stack.ts";
import { Stack } from "./stack.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: StackExpose;

type _AxisIsLogical = Expect<Equal<StackAxis, "block" | "inline">>;
type _AlignIsLogical = Expect<
  Equal<StackAlign, "stretch" | "start" | "center" | "end" | "baseline">
>;
type _JustifyIsLogical = Expect<
  Equal<
    StackJustify,
    "start" | "center" | "end" | "space-between" | "space-around" | "space-evenly"
  >
>;
type _DirectionIsFlexLiteral = Expect<
  Equal<StackFlexDirection, "column" | "column-reverse" | "row" | "row-reverse">
>;
type _ElementIsRenderable = Expect<Equal<StackElement, Element | ComponentPublicInstance>>;
type _GapIsCssString = Expect<Equal<StackGap, string>>;
type _StateIsStacked = Expect<Equal<StackState, "stacked">>;
type _SlotStateIsStable = Expect<
  Equal<
    StackSlotState,
    {
      readonly axis: StackAxis;
      readonly reversed: boolean;
      readonly direction: StackFlexDirection;
      readonly gap: StackGap;
      readonly align: StackAlign;
      readonly justify: StackJustify;
      readonly state: StackState;
    }
  >
>;
type _ExposeStateMatchesSlot = Expect<
  Equal<
    Omit<StackExpose, "element">,
    {
      readonly axis: StackAxis;
      readonly reversed: boolean;
      readonly direction: StackFlexDirection;
      readonly gap: StackGap;
      readonly align: StackAlign;
      readonly justify: StackJustify;
      readonly state: StackState;
    }
  >
>;
type _StyleIsFlexOnly = Expect<
  Equal<
    StackStyle,
    {
      readonly "--vize-ui-stack-gap": StackGap;
      readonly "--vize-ui-stack-align": StackAlign;
      readonly "--vize-ui-stack-justify": StackJustify;
      readonly display: "flex";
      readonly flexDirection: StackFlexDirection;
      readonly gap: string;
      readonly alignItems: string;
      readonly justifyContent: string;
    }
  >
>;

const exposedElement: StackElement | null = exposed.element;
const resolved: StackResolvedLayout = {
  align: "center",
  axis: "inline",
  direction: "row-reverse",
  gap: "1lh",
  justify: "space-between",
  reversed: true,
  state: "stacked",
  style: {
    "--vize-ui-stack-align": "center",
    "--vize-ui-stack-gap": "1lh",
    "--vize-ui-stack-justify": "space-between",
    alignItems: "var(--vize-ui-stack-align)",
    display: "flex",
    flexDirection: "row-reverse",
    gap: "var(--vize-ui-stack-gap)",
    justifyContent: "var(--vize-ui-stack-justify)",
  },
};
const customHost: InstanceType<typeof Stack>["$props"] = {
  align: "baseline",
  as: componentTarget,
  axis: "inline",
  gap: "var(--space-4)",
  justify: "space-evenly",
  reversed: true,
};

// @ts-expect-error axis is intentionally logical; physical horizontal/vertical names belong in user CSS.
const badAxis: StackAxis = "horizontal";

// @ts-expect-error gap values must be native CSS strings with explicit units, keywords, or variables.
const badGap: InstanceType<typeof Stack>["$props"] = { gap: 2 };

// @ts-expect-error alignment uses logical CSS values, not physical left/right.
const badAlign: InstanceType<typeof Stack>["$props"] = { align: "left" };

// @ts-expect-error justification uses native CSS keywords, not component-specific aliases.
const badJustify: InstanceType<typeof Stack>["$props"] = { justify: "between" };

void Stack;
void badAlign;
void badAxis;
void badGap;
void badJustify;
void customHost;
void exposed;
void exposedElement;
void resolved;
