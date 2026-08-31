/** Compile-only assertions for the public Icon and IconButton contracts. */

import type { Component, ComponentPublicInstance } from "vue";

import { Icon } from "./icon.ts";
import { IconButton } from "./icon-button.ts";
import type {
  IconAriaState,
  IconElement,
  IconExpose,
  IconPaint,
  IconProps,
  IconSize,
  IconSlotState,
  IconStrokeLinecap,
  IconStrokeLinejoin,
  IconStrokeWidth,
} from "./icon.ts";
import type {
  IconButtonElement,
  IconButtonExpose,
  IconButtonProps,
  IconButtonSize,
  IconButtonSlotState,
  IconButtonState,
  IconButtonTone,
  IconButtonType,
  IconButtonVariant,
} from "./icon-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const iconExposed: IconExpose;
declare const iconButtonExposed: IconButtonExpose;

type _IconAriaStateIsLiteral = Expect<Equal<IconAriaState, "decorative" | "img">>;
type _IconSizeIsLiteral = Expect<Equal<IconSize, "xs" | "sm" | "md" | "lg" | "xl">>;
type _IconPaintIsLiteral = Expect<Equal<IconPaint, "currentColor" | "none">>;
type _IconLinecapIsLiteral = Expect<Equal<IconStrokeLinecap, "butt" | "round" | "square">>;
type _IconLinejoinIsLiteral = Expect<
  Equal<IconStrokeLinejoin, "arcs" | "bevel" | "miter" | "miter-clip" | "round">
>;
type _IconElementIsRenderable = Expect<Equal<IconElement, Element | ComponentPublicInstance>>;
type _IconPropsFeedComponentProps = Expect<
  IconProps extends InstanceType<typeof Icon>["$props"] ? true : false
>;
type _IconSlotStateIsLiteral = Expect<
  Equal<
    IconSlotState,
    {
      readonly ariaState: IconAriaState;
      readonly decorative: boolean;
      readonly descriptionId: string | undefined;
      readonly size: IconSize;
      readonly titleId: string | undefined;
      readonly viewBox: string;
    }
  >
>;
type _IconExposeStateIsLiteral = Expect<Equal<typeof iconExposed.ariaState, IconAriaState>>;
type _IconExposeElementIsNullable = Expect<Equal<typeof iconExposed.element, IconElement | null>>;

type _IconButtonTypeIsLiteral = Expect<Equal<IconButtonType, "button" | "reset" | "submit">>;
type _IconButtonSizeIsLiteral = Expect<Equal<IconButtonSize, "sm" | "md" | "lg">>;
type _IconButtonToneIsLiteral = Expect<Equal<IconButtonTone, "accent" | "danger" | "neutral">>;
type _IconButtonVariantIsLiteral = Expect<
  Equal<IconButtonVariant, "outline" | "plain" | "soft" | "solid">
>;
type _IconButtonStateIsLiteral = Expect<Equal<IconButtonState, "disabled" | "idle" | "loading">>;
type _IconButtonElementIsRenderable = Expect<
  Equal<IconButtonElement, Element | ComponentPublicInstance>
>;
type _IconButtonPropsFeedComponentProps = Expect<
  IconButtonProps extends InstanceType<typeof IconButton>["$props"] ? true : false
>;
type _IconButtonRequiresName = Expect<Equal<{} extends IconButtonProps ? true : false, false>>;
type _IconButtonSlotStateIsLiteral = Expect<
  Equal<
    IconButtonSlotState,
    {
      readonly disabled: boolean;
      readonly loading: boolean;
      readonly size: IconButtonSize;
      readonly state: IconButtonState;
      readonly tone: IconButtonTone;
      readonly unavailable: boolean;
      readonly variant: IconButtonVariant;
    }
  >
>;
type _IconButtonExposeStateIsLiteral = Expect<
  Equal<typeof iconButtonExposed.state, IconButtonState>
>;
type _IconButtonExposeElementIsNullable = Expect<
  Equal<typeof iconButtonExposed.element, IconButtonElement | null>
>;

const decorativeIcon: IconProps = {};
const labelledIcon: IconProps = {
  ariaLabel: "Search",
  as: componentTarget,
  fill: "currentColor",
  focusable: false,
  size: "sm",
  stroke: "none",
  strokeLinecap: "butt",
  strokeLinejoin: "miter-clip",
  strokeWidth: "1.5",
};
const titleIcon: IconProps = {
  description: "Reloads the current feed",
  descriptionId: "refresh-icon-description",
  title: "Refresh",
  titleId: "refresh-icon-title",
};
const iconButtonProps: IconButtonProps = {
  ariaLabel: "Refresh",
  size: "md",
  tone: "accent",
  type: "button",
  variant: "soft",
};
const labelledIconButtonProps: IconButtonProps = {
  ariaLabelledby: "refresh-label",
  as: componentTarget,
};
const buttonInstanceProps: InstanceType<typeof IconButton>["$props"] = {
  ariaLabel: "Settings",
  variant: "plain",
};
const width: IconStrokeWidth = 2;
const stringWidth: IconStrokeWidth = "2.5";

// @ts-expect-error non-decorative icons require an accessible name.
const unlabeledImageIcon: IconProps = { decorative: false };

// @ts-expect-error hidden decorative icons cannot also carry a direct name in the strict API.
const hiddenLabelledIcon: IconProps = { ariaHidden: true, ariaLabel: "Hidden" };

// @ts-expect-error root paint tokens are intentionally closed.
const badPaint: IconPaint = "red";

// @ts-expect-error stroke linecaps are native SVG literal tokens.
const badLinecap: IconStrokeLinecap = "center";

// @ts-expect-error stroke linejoins are native SVG literal tokens.
const badLinejoin: IconStrokeLinejoin = "curve";

// @ts-expect-error stroke widths reject arbitrary strings.
const badStrokeWidth: IconStrokeWidth = "var(--icon-stroke-width)";

// @ts-expect-error icon sizes are strict consumer styling tokens.
const badIconSize: IconSize = "2xl";

// @ts-expect-error icon-only buttons require an accessible name.
const missingButtonName: IconButtonProps = {};

// @ts-expect-error icon-button variants are strict tokens.
const badVariant: IconButtonVariant = "ghost";

// @ts-expect-error icon-button tones are intentionally narrow.
const badTone: IconButtonTone = "success";

// @ts-expect-error native button type is constrained to HTML button types.
const badButtonType: IconButtonProps = { ariaLabel: "Go", type: "link" };

void Icon;
void IconButton;
void badButtonType;
void badIconSize;
void badLinejoin;
void badLinecap;
void badPaint;
void badStrokeWidth;
void badTone;
void badVariant;
void buttonInstanceProps;
void decorativeIcon;
void hiddenLabelledIcon;
void iconButtonProps;
void labelledIcon;
void labelledIconButtonProps;
void missingButtonName;
void stringWidth;
void titleIcon;
void unlabeledImageIcon;
void width;
