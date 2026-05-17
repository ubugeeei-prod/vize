/**
 * Box Component - Container with flexbox layout
 */

import { defineComponent, h, type PropType } from "@vue/runtime-core";

export type DimensionValue = number | string;
export type BorderStyleName =
  | "none"
  | "single"
  | "double"
  | "round"
  | "rounded"
  | "bold"
  | "heavy"
  | "dashed";

export interface BoxProps {
  /** Display type */
  display?: "flex" | "none";
  /** Positioning mode */
  position?: "absolute" | "relative" | "static";
  top?: DimensionValue;
  right?: DimensionValue;
  bottom?: DimensionValue;
  left?: DimensionValue;
  /** Flex direction */
  flexDirection?: "row" | "column" | "row-reverse" | "column-reverse";
  /** Flex wrap */
  flexWrap?: "nowrap" | "wrap" | "wrap-reverse";
  /** Justify content */
  justifyContent?:
    | "flex-start"
    | "flex-end"
    | "center"
    | "space-between"
    | "space-around"
    | "space-evenly";
  /** Align items */
  alignItems?: "flex-start" | "flex-end" | "center" | "stretch" | "baseline";
  /** Align self */
  alignSelf?: "auto" | "flex-start" | "flex-end" | "center" | "stretch" | "baseline";
  /** Align content */
  alignContent?:
    | "flex-start"
    | "flex-end"
    | "center"
    | "stretch"
    | "space-between"
    | "space-around"
    | "space-evenly";
  /** Flex grow */
  flexGrow?: number;
  /** Flex shrink */
  flexShrink?: number;
  /** Flex basis */
  flexBasis?: DimensionValue;
  /** Width */
  width?: DimensionValue;
  /** Height */
  height?: DimensionValue;
  /** Min width */
  minWidth?: DimensionValue;
  /** Min height */
  minHeight?: DimensionValue;
  /** Max width */
  maxWidth?: DimensionValue;
  /** Max height */
  maxHeight?: DimensionValue;
  /** Aspect ratio */
  aspectRatio?: number;
  /** Padding (all sides) */
  padding?: number;
  /** Padding X (left and right) */
  paddingX?: number;
  /** Padding Y (top and bottom) */
  paddingY?: number;
  /** Padding top */
  paddingTop?: number;
  /** Padding right */
  paddingRight?: number;
  /** Padding bottom */
  paddingBottom?: number;
  /** Padding left */
  paddingLeft?: number;
  /** Margin (all sides) */
  margin?: number;
  /** Margin X (left and right) */
  marginX?: number;
  /** Margin Y (top and bottom) */
  marginY?: number;
  /** Margin top */
  marginTop?: number;
  /** Margin right */
  marginRight?: number;
  /** Margin bottom */
  marginBottom?: number;
  /** Margin left */
  marginLeft?: number;
  /** Gap between children */
  gap?: number;
  /** Column gap between children */
  columnGap?: number;
  /** Row gap between children */
  rowGap?: number;
  /** Overflow behavior */
  overflow?: "visible" | "hidden" | "scroll";
  overflowX?: "visible" | "hidden" | "scroll";
  overflowY?: "visible" | "hidden" | "scroll";
  /** Border style (Fresco alias) */
  border?: BorderStyleName;
  /** Border style (Ink alias) */
  borderStyle?: BorderStyleName;
  borderTop?: boolean;
  borderRight?: boolean;
  borderBottom?: boolean;
  borderLeft?: boolean;
  borderColor?: string;
  borderTopColor?: string;
  borderRightColor?: string;
  borderBottomColor?: string;
  borderLeftColor?: string;
  borderDimColor?: boolean;
  borderTopDimColor?: boolean;
  borderRightDimColor?: boolean;
  borderBottomDimColor?: boolean;
  borderLeftDimColor?: boolean;
  borderBackgroundColor?: string;
  borderTopBackgroundColor?: string;
  borderRightBackgroundColor?: string;
  borderBottomBackgroundColor?: string;
  borderLeftBackgroundColor?: string;
  /** Foreground color */
  fg?: string;
  /** Foreground color alias */
  color?: string;
  /** Background color */
  bg?: string;
  /** Ink-compatible background color */
  backgroundColor?: string;
  /** Accessibility label, accepted for Ink API parity */
  "aria-label"?: string;
  /** Hide from screen readers, accepted for Ink API parity */
  "aria-hidden"?: boolean;
  /** Accessibility role, accepted for Ink API parity */
  "aria-role"?:
    | "button"
    | "checkbox"
    | "combobox"
    | "list"
    | "listbox"
    | "listitem"
    | "menu"
    | "menuitem"
    | "option"
    | "progressbar"
    | "radio"
    | "radiogroup"
    | "tab"
    | "tablist"
    | "table"
    | "textbox"
    | "timer"
    | "toolbar";
  /** Accessibility state, accepted for Ink API parity */
  "aria-state"?: Record<string, boolean | undefined>;
}

const dimensionProp = [Number, String] as PropType<DimensionValue>;

function normalizeBorderStyle(style: BorderStyleName | undefined): BorderStyleName | undefined {
  if (style === "round") return "rounded";
  if (style === "bold") return "heavy";
  return style;
}

export const Box = defineComponent({
  name: "Box",
  props: {
    display: String as PropType<BoxProps["display"]>,
    position: String as PropType<BoxProps["position"]>,
    top: dimensionProp,
    right: dimensionProp,
    bottom: dimensionProp,
    left: dimensionProp,
    flexDirection: String as PropType<BoxProps["flexDirection"]>,
    flexWrap: String as PropType<BoxProps["flexWrap"]>,
    justifyContent: String as PropType<BoxProps["justifyContent"]>,
    alignItems: String as PropType<BoxProps["alignItems"]>,
    alignSelf: String as PropType<BoxProps["alignSelf"]>,
    alignContent: String as PropType<BoxProps["alignContent"]>,
    flexGrow: Number,
    flexShrink: Number,
    flexBasis: dimensionProp,
    width: dimensionProp,
    height: dimensionProp,
    minWidth: dimensionProp,
    minHeight: dimensionProp,
    maxWidth: dimensionProp,
    maxHeight: dimensionProp,
    aspectRatio: Number,
    padding: Number,
    paddingX: Number,
    paddingY: Number,
    paddingTop: Number,
    paddingRight: Number,
    paddingBottom: Number,
    paddingLeft: Number,
    margin: Number,
    marginX: Number,
    marginY: Number,
    marginTop: Number,
    marginRight: Number,
    marginBottom: Number,
    marginLeft: Number,
    gap: Number,
    columnGap: Number,
    rowGap: Number,
    overflow: String as PropType<BoxProps["overflow"]>,
    overflowX: String as PropType<BoxProps["overflowX"]>,
    overflowY: String as PropType<BoxProps["overflowY"]>,
    border: String as PropType<BoxProps["border"]>,
    borderStyle: String as PropType<BoxProps["borderStyle"]>,
    borderTop: Boolean,
    borderRight: Boolean,
    borderBottom: Boolean,
    borderLeft: Boolean,
    borderColor: String,
    borderTopColor: String,
    borderRightColor: String,
    borderBottomColor: String,
    borderLeftColor: String,
    borderDimColor: Boolean,
    borderTopDimColor: Boolean,
    borderRightDimColor: Boolean,
    borderBottomDimColor: Boolean,
    borderLeftDimColor: Boolean,
    borderBackgroundColor: String,
    borderTopBackgroundColor: String,
    borderRightBackgroundColor: String,
    borderBottomBackgroundColor: String,
    borderLeftBackgroundColor: String,
    fg: String,
    color: String,
    bg: String,
    backgroundColor: String,
    "aria-label": String,
    "aria-hidden": Boolean,
    "aria-role": String as PropType<BoxProps["aria-role"]>,
    "aria-state": Object as PropType<BoxProps["aria-state"]>,
  },
  setup(props, { slots }) {
    return () => {
      const style: Record<string, unknown> = {};

      if (props.display) style.display = props.display;
      if (props.position) style.position = props.position;
      if (props.top !== undefined) style.top = String(props.top);
      if (props.right !== undefined) style.right = String(props.right);
      if (props.bottom !== undefined) style.bottom = String(props.bottom);
      if (props.left !== undefined) style.left = String(props.left);

      if (props.flexDirection) style.flexDirection = props.flexDirection;
      if (props.flexWrap) style.flexWrap = props.flexWrap;
      if (props.justifyContent) style.justifyContent = props.justifyContent;
      if (props.alignItems) style.alignItems = props.alignItems;
      if (props.alignSelf) style.alignSelf = props.alignSelf;
      if (props.alignContent) style.alignContent = props.alignContent;
      if (props.flexGrow !== undefined) style.flexGrow = props.flexGrow;
      if (props.flexShrink !== undefined) style.flexShrink = props.flexShrink;
      if (props.flexBasis !== undefined) style.flexBasis = String(props.flexBasis);

      if (props.width !== undefined) style.width = String(props.width);
      if (props.height !== undefined) style.height = String(props.height);
      if (props.minWidth !== undefined) style.minWidth = String(props.minWidth);
      if (props.minHeight !== undefined) style.minHeight = String(props.minHeight);
      if (props.maxWidth !== undefined) style.maxWidth = String(props.maxWidth);
      if (props.maxHeight !== undefined) style.maxHeight = String(props.maxHeight);
      if (props.aspectRatio !== undefined) style.aspectRatio = props.aspectRatio;

      if (props.padding !== undefined) style.padding = props.padding;
      if (props.paddingTop !== undefined || props.paddingY !== undefined) {
        style.paddingTop = props.paddingTop ?? props.paddingY ?? props.padding;
      }
      if (props.paddingRight !== undefined || props.paddingX !== undefined) {
        style.paddingRight = props.paddingRight ?? props.paddingX ?? props.padding;
      }
      if (props.paddingBottom !== undefined || props.paddingY !== undefined) {
        style.paddingBottom = props.paddingBottom ?? props.paddingY ?? props.padding;
      }
      if (props.paddingLeft !== undefined || props.paddingX !== undefined) {
        style.paddingLeft = props.paddingLeft ?? props.paddingX ?? props.padding;
      }

      if (props.margin !== undefined) style.margin = props.margin;
      if (props.marginTop !== undefined || props.marginY !== undefined) {
        style.marginTop = props.marginTop ?? props.marginY ?? props.margin;
      }
      if (props.marginRight !== undefined || props.marginX !== undefined) {
        style.marginRight = props.marginRight ?? props.marginX ?? props.margin;
      }
      if (props.marginBottom !== undefined || props.marginY !== undefined) {
        style.marginBottom = props.marginBottom ?? props.marginY ?? props.margin;
      }
      if (props.marginLeft !== undefined || props.marginX !== undefined) {
        style.marginLeft = props.marginLeft ?? props.marginX ?? props.margin;
      }

      if (props.gap !== undefined) style.gap = props.gap;
      if (props.columnGap !== undefined) style.columnGap = props.columnGap;
      if (props.rowGap !== undefined) style.rowGap = props.rowGap;
      if (props.overflow) style.overflow = props.overflow;
      if (props.overflowX) style.overflowX = props.overflowX;
      if (props.overflowY) style.overflowY = props.overflowY;

      return h(
        "box",
        {
          style,
          border: normalizeBorderStyle(props.borderStyle ?? props.border),
          borderColor: props.borderColor,
          borderTopColor: props.borderTopColor,
          borderRightColor: props.borderRightColor,
          borderBottomColor: props.borderBottomColor,
          borderLeftColor: props.borderLeftColor,
          borderDimColor: props.borderDimColor,
          borderBackgroundColor: props.borderBackgroundColor,
          fg: props.fg ?? props.color,
          bg: props.bg ?? props.backgroundColor,
          "aria-label": props["aria-label"],
          "aria-hidden": props["aria-hidden"],
          "aria-role": props["aria-role"],
          "aria-state": props["aria-state"],
        },
        slots.default?.(),
      );
    };
  },
});
