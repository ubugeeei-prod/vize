/** Compile-only assertions for the public container contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  ContainerElement,
  ContainerExpose,
  ContainerLength,
  ContainerResolvedLayout,
  ContainerResolvedLength,
  ContainerSize,
  ContainerSlotState,
  ContainerStyle,
} from "./container.ts";
import { Container } from "./container.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: ContainerExpose;

type _SizeIsNamedScale = Expect<Equal<ContainerSize, "xs" | "sm" | "md" | "lg" | "xl" | "full">>;
type _ElementIsRenderable = Expect<Equal<ContainerElement, Element | ComponentPublicInstance>>;
type _LengthAcceptsCssStringOrNumber = Expect<Equal<ContainerLength, string | number>>;
type _ResolvedLengthIsCssString = Expect<Equal<ContainerResolvedLength, string>>;
type _StyleUsesLogicalProperties = Expect<
  Equal<
    ContainerStyle,
    {
      readonly "--vize-ui-container-max-inline-size": ContainerResolvedLength;
      readonly "--vize-ui-container-padding-inline": ContainerResolvedLength;
      readonly maxInlineSize: string;
      readonly paddingInline: string;
      readonly marginInline?: "auto";
    }
  >
>;
type _SlotStateIsStable = Expect<
  Equal<
    ContainerSlotState,
    {
      readonly size: ContainerSize;
      readonly maxInlineSize: ContainerResolvedLength;
      readonly paddingInline: ContainerResolvedLength;
      readonly centered: boolean;
      readonly style: ContainerStyle;
    }
  >
>;
type _ExposeStateMatchesSlot = Expect<
  Equal<
    Omit<ContainerExpose, "element">,
    {
      readonly size: ContainerSize;
      readonly maxInlineSize: ContainerResolvedLength;
      readonly paddingInline: ContainerResolvedLength;
      readonly centered: boolean;
      readonly style: ContainerStyle;
    }
  >
>;

const exposedElement: ContainerElement | null = exposed.element;
const resolved: ContainerResolvedLayout = {
  centered: true,
  maxInlineSize: "64rem",
  paddingInline: "1rem",
  size: "md",
  style: {
    "--vize-ui-container-max-inline-size": "64rem",
    "--vize-ui-container-padding-inline": "1rem",
    marginInline: "auto",
    maxInlineSize: "var(--vize-ui-container-max-inline-size)",
    paddingInline: "var(--vize-ui-container-padding-inline)",
  },
};
const customHost: InstanceType<typeof Container>["$props"] = {
  as: componentTarget,
  centered: false,
  maxInlineSize: 960,
  paddingInline: "2rem",
  size: "xl",
};

// @ts-expect-error size is constrained to the named container scale.
const badSize: InstanceType<typeof Container>["$props"] = { size: "2xl" };

// @ts-expect-error maxInlineSize values must be CSS strings or numeric pixel lengths.
const badMaxInlineSize: InstanceType<typeof Container>["$props"] = { maxInlineSize: true };

// @ts-expect-error paddingInline values must be CSS strings or numeric pixel lengths.
const badPaddingInline: InstanceType<typeof Container>["$props"] = { paddingInline: false };

// @ts-expect-error centered is a boolean logical margin flag.
const badCentered: InstanceType<typeof Container>["$props"] = { centered: "true" };

void Container;
void badCentered;
void badMaxInlineSize;
void badPaddingInline;
void badSize;
void customHost;
void exposed;
void exposedElement;
void resolved;
