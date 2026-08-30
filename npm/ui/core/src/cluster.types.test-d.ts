/** Compile-only assertions for the public cluster contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  ClusterAlign,
  ClusterElement,
  ClusterExpose,
  ClusterFlexDirection,
  ClusterFlexWrap,
  ClusterGap,
  ClusterJustify,
  ClusterResolvedGap,
  ClusterResolvedLayout,
  ClusterSlotState,
  ClusterState,
  ClusterStyle,
} from "./cluster.ts";
import { Cluster } from "./cluster.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: ClusterExpose;

type _AlignIsLogical = Expect<
  Equal<ClusterAlign, "stretch" | "start" | "center" | "end" | "baseline">
>;
type _JustifyIsLogical = Expect<
  Equal<
    ClusterJustify,
    "start" | "center" | "end" | "space-between" | "space-around" | "space-evenly"
  >
>;
type _DirectionIsInlineFlexLiteral = Expect<Equal<ClusterFlexDirection, "row" | "row-reverse">>;
type _WrapModeIsFlexLiteral = Expect<Equal<ClusterFlexWrap, "wrap" | "nowrap">>;
type _ElementIsRenderable = Expect<Equal<ClusterElement, Element | ComponentPublicInstance>>;
type _GapAcceptsCssStringOrNumber = Expect<Equal<ClusterGap, string | number>>;
type _ResolvedGapIsCssString = Expect<Equal<ClusterResolvedGap, string>>;
type _StateIsClustered = Expect<Equal<ClusterState, "clustered">>;
type _SlotStateIsStable = Expect<
  Equal<
    ClusterSlotState,
    {
      readonly wrap: boolean;
      readonly reversed: boolean;
      readonly direction: ClusterFlexDirection;
      readonly wrapMode: ClusterFlexWrap;
      readonly gap: ClusterResolvedGap;
      readonly align: ClusterAlign;
      readonly justify: ClusterJustify;
      readonly state: ClusterState;
    }
  >
>;
type _ExposeStateMatchesSlot = Expect<
  Equal<
    Omit<ClusterExpose, "element">,
    {
      readonly wrap: boolean;
      readonly reversed: boolean;
      readonly direction: ClusterFlexDirection;
      readonly wrapMode: ClusterFlexWrap;
      readonly gap: ClusterResolvedGap;
      readonly align: ClusterAlign;
      readonly justify: ClusterJustify;
      readonly state: ClusterState;
    }
  >
>;
type _StyleIsWrappingFlexOnly = Expect<
  Equal<
    ClusterStyle,
    {
      readonly "--vize-ui-cluster-gap": ClusterResolvedGap;
      readonly "--vize-ui-cluster-align": ClusterAlign;
      readonly "--vize-ui-cluster-justify": ClusterJustify;
      readonly display: "flex";
      readonly flexDirection: ClusterFlexDirection;
      readonly flexWrap: ClusterFlexWrap;
      readonly gap: string;
      readonly alignItems: string;
      readonly justifyContent: string;
    }
  >
>;

const exposedElement: ClusterElement | null = exposed.element;
const resolved: ClusterResolvedLayout = {
  align: "center",
  direction: "row-reverse",
  gap: "1lh",
  justify: "space-between",
  reversed: true,
  state: "clustered",
  wrap: false,
  wrapMode: "nowrap",
  style: {
    "--vize-ui-cluster-align": "center",
    "--vize-ui-cluster-gap": "1lh",
    "--vize-ui-cluster-justify": "space-between",
    alignItems: "var(--vize-ui-cluster-align)",
    display: "flex",
    flexDirection: "row-reverse",
    flexWrap: "nowrap",
    gap: "var(--vize-ui-cluster-gap)",
    justifyContent: "var(--vize-ui-cluster-justify)",
  },
};
const customHost: InstanceType<typeof Cluster>["$props"] = {
  align: "baseline",
  as: componentTarget,
  gap: 8,
  justify: "space-evenly",
  reversed: true,
  wrap: false,
};

// @ts-expect-error alignment uses logical CSS values, not physical left/right.
const badAlign: InstanceType<typeof Cluster>["$props"] = { align: "left" };

// @ts-expect-error justification uses native CSS keywords, not component-specific aliases.
const badJustify: InstanceType<typeof Cluster>["$props"] = { justify: "between" };

// @ts-expect-error wrap is a boolean layout flag.
const badWrap: InstanceType<typeof Cluster>["$props"] = { wrap: "wrap" };

// @ts-expect-error gap values must be CSS strings or numeric pixel lengths.
const badGap: InstanceType<typeof Cluster>["$props"] = { gap: true };

void Cluster;
void badAlign;
void badGap;
void badJustify;
void badWrap;
void customHost;
void exposed;
void exposedElement;
void resolved;
