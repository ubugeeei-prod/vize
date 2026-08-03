import type {
  FrescoAlignContent,
  FrescoAlignItems,
  FrescoAlignSelf,
  FrescoAppearance,
  FrescoBorderStyle,
  FrescoDisplay,
  FrescoFlexDirection,
  FrescoFlexWrap,
  FrescoInputRenderNode,
  FrescoJustifyContent,
  FrescoOverflow,
  FrescoPosition,
  FrescoRenderNode,
  FrescoRenderStyle,
  FrescoRootRenderNode,
  FrescoTextRenderNode,
  FrescoTextWrapMode,
} from "./protocol.js";
import type { FrescoNode } from "./renderer.js";

const displays = ["flex", "none"] as const satisfies readonly FrescoDisplay[];
const positions = ["absolute", "relative", "static"] as const satisfies readonly FrescoPosition[];
const overflows = ["visible", "hidden", "scroll"] as const satisfies readonly FrescoOverflow[];
const flexDirections = [
  "row",
  "column",
  "row-reverse",
  "column-reverse",
] as const satisfies readonly FrescoFlexDirection[];
const flexWraps = ["nowrap", "wrap", "wrap-reverse"] as const satisfies readonly FrescoFlexWrap[];
const justifyContents = [
  "flex-start",
  "flex-end",
  "start",
  "end",
  "center",
  "space-between",
  "space-around",
  "space-evenly",
] as const satisfies readonly FrescoJustifyContent[];
const alignItems = [
  "flex-start",
  "flex-end",
  "start",
  "end",
  "center",
  "stretch",
  "baseline",
] as const satisfies readonly FrescoAlignItems[];
const alignSelf = ["auto", ...alignItems] as const satisfies readonly FrescoAlignSelf[];
const alignContent = [
  "flex-start",
  "flex-end",
  "start",
  "end",
  "center",
  "stretch",
  "space-between",
  "space-around",
  "space-evenly",
] as const satisfies readonly FrescoAlignContent[];
const borderStyles = [
  "none",
  "single",
  "double",
  "rounded",
  "heavy",
  "dashed",
] as const satisfies readonly FrescoBorderStyle[];
const wrapModes = [
  "none",
  "wrap",
  "hard",
  "truncate",
  "truncate-start",
  "truncate-middle",
  "truncate-end",
] as const satisfies readonly FrescoTextWrapMode[];

function recordValue(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null ? (value as Record<string, unknown>) : {};
}

function firstValue(record: Record<string, unknown>, ...keys: string[]): unknown {
  for (const key of keys) {
    if (record[key] !== undefined) return record[key];
  }
  return undefined;
}

function enumValue<Value extends string>(
  value: unknown,
  values: readonly Value[],
): Value | undefined {
  return typeof value === "string" && values.includes(value as Value)
    ? (value as Value)
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" || typeof value === "number" ? String(value) : undefined;
}

function numberValue(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function booleanValue(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

function enabledValue(value: unknown): true | undefined {
  return value === true ? true : undefined;
}

function nonEmptyStringValue(value: unknown): string | undefined {
  const string = stringValue(value);
  return string ? string : undefined;
}

export function normalizeFrescoStyle(value: unknown): FrescoRenderStyle | undefined {
  const source = recordValue(value);
  const style: FrescoRenderStyle = {};

  style.display = enumValue(source.display, displays);
  style.position = enumValue(source.position, positions);
  style.top = stringValue(source.top);
  style.right = stringValue(source.right);
  style.bottom = stringValue(source.bottom);
  style.left = stringValue(source.left);
  style.overflow = enumValue(source.overflow, overflows);
  style.overflowX = enumValue(firstValue(source, "overflowX", "overflow_x"), overflows);
  style.overflowY = enumValue(firstValue(source, "overflowY", "overflow_y"), overflows);
  style.flexDirection = enumValue(
    firstValue(source, "flexDirection", "flex_direction"),
    flexDirections,
  );
  style.flexWrap = enumValue(firstValue(source, "flexWrap", "flex_wrap"), flexWraps);
  style.justifyContent = enumValue(
    firstValue(source, "justifyContent", "justify_content"),
    justifyContents,
  );
  style.alignItems = enumValue(firstValue(source, "alignItems", "align_items"), alignItems);
  style.alignSelf = enumValue(firstValue(source, "alignSelf", "align_self"), alignSelf);
  style.alignContent = enumValue(firstValue(source, "alignContent", "align_content"), alignContent);
  style.flexGrow = numberValue(firstValue(source, "flexGrow", "flex_grow"));
  style.flexShrink = numberValue(firstValue(source, "flexShrink", "flex_shrink"));
  style.flexBasis = stringValue(firstValue(source, "flexBasis", "flex_basis"));
  style.width = stringValue(source.width);
  style.height = stringValue(source.height);
  style.minWidth = stringValue(firstValue(source, "minWidth", "min_width"));
  style.minHeight = stringValue(firstValue(source, "minHeight", "min_height"));
  style.maxWidth = stringValue(firstValue(source, "maxWidth", "max_width"));
  style.maxHeight = stringValue(firstValue(source, "maxHeight", "max_height"));
  style.aspectRatio = numberValue(firstValue(source, "aspectRatio", "aspect_ratio"));
  style.padding = numberValue(source.padding);
  style.paddingTop = numberValue(firstValue(source, "paddingTop", "padding_top"));
  style.paddingRight = numberValue(firstValue(source, "paddingRight", "padding_right"));
  style.paddingBottom = numberValue(firstValue(source, "paddingBottom", "padding_bottom"));
  style.paddingLeft = numberValue(firstValue(source, "paddingLeft", "padding_left"));
  style.margin = numberValue(source.margin);
  style.marginTop = numberValue(firstValue(source, "marginTop", "margin_top"));
  style.marginRight = numberValue(firstValue(source, "marginRight", "margin_right"));
  style.marginBottom = numberValue(firstValue(source, "marginBottom", "margin_bottom"));
  style.marginLeft = numberValue(firstValue(source, "marginLeft", "margin_left"));
  style.gap = numberValue(source.gap);
  style.columnGap = numberValue(firstValue(source, "columnGap", "column_gap"));
  style.rowGap = numberValue(firstValue(source, "rowGap", "row_gap"));

  for (const key of Object.keys(style) as (keyof FrescoRenderStyle)[]) {
    if (style[key] === undefined) delete style[key];
  }
  return Object.keys(style).length > 0 ? style : undefined;
}

function normalizeAppearance(props: Record<string, unknown>): FrescoAppearance | undefined {
  const appearance: FrescoAppearance = {};
  appearance.fg = nonEmptyStringValue(firstValue(props, "fg", "color"));
  appearance.bg = nonEmptyStringValue(
    firstValue(props, "bg", "backgroundColor", "background_color"),
  );
  appearance.bold = enabledValue(props.bold);
  appearance.dim = enabledValue(firstValue(props, "dim", "dimColor", "dim_color"));
  appearance.italic = enabledValue(props.italic);
  appearance.underline = enabledValue(props.underline);
  appearance.inverse = enabledValue(props.inverse);
  appearance.blink = enabledValue(props.blink);
  appearance.hidden = enabledValue(props.hidden);
  appearance.strikethrough = enabledValue(props.strikethrough);

  for (const key of Object.keys(appearance) as (keyof FrescoAppearance)[]) {
    if (appearance[key] === undefined) delete appearance[key];
  }
  return Object.keys(appearance).length > 0 ? appearance : undefined;
}

function commonFields(node: FrescoNode): Omit<FrescoRootRenderNode, "nodeType"> {
  const common: Omit<FrescoRootRenderNode, "nodeType"> = { id: node.id };
  const style = normalizeFrescoStyle(node.props.style);
  const appearance = normalizeAppearance(node.props);
  const border = enumValue(node.props.border, borderStyles);
  if (style) common.style = style;
  if (appearance) common.appearance = appearance;
  if (border) common.border = border;
  if (node.children.length > 0) common.children = node.children.map((child) => child.id);
  return common;
}

function wrapMode(value: unknown): FrescoTextWrapMode | undefined {
  if (value === false) return "none";
  if (value === true) return "wrap";
  if (value === "end") return "truncate-end";
  if (value === "middle") return "truncate-middle";
  return enumValue(value, wrapModes);
}

function wrappingEnabled(value: unknown, mode: FrescoTextWrapMode | undefined): boolean {
  if (value === undefined || value === false) return false;
  return !(mode?.startsWith("truncate") ?? false);
}

export function frescoNodeToRenderNode(node: FrescoNode): FrescoRenderNode {
  const common = commonFields(node);

  if (node.type === "root" || node.type === "box") {
    return { ...common, nodeType: node.type };
  }

  if (node.type === "text") {
    const renderNode: FrescoTextRenderNode = { ...common, nodeType: "text" };
    const text = node.text ?? stringValue(firstValue(node.props, "text", "content"));
    const mode = wrapMode(node.props.wrap);
    if (text !== undefined) renderNode.text = text;
    if (node.props.wrap !== undefined) {
      renderNode.wrap = wrappingEnabled(node.props.wrap, mode);
      if (mode) renderNode.wrapMode = mode;
    }
    return renderNode;
  }

  const renderNode: FrescoInputRenderNode = { ...common, nodeType: "input" };
  const value = stringValue(node.props.value);
  const placeholder = stringValue(node.props.placeholder);
  const focused = booleanValue(firstValue(node.props, "focused", "focus"));
  const cursor = numberValue(node.props.cursor);
  const mask = booleanValue(node.props.mask);
  const maskChar = stringValue(firstValue(node.props, "maskChar", "mask_char", "mask-char"));
  if (value !== undefined) renderNode.value = value;
  if (placeholder !== undefined) renderNode.placeholder = placeholder;
  if (focused !== undefined) renderNode.focused = focused;
  if (cursor !== undefined) renderNode.cursor = cursor;
  if (mask !== undefined) renderNode.mask = mask;
  if (maskChar !== undefined) renderNode.maskChar = maskChar;
  return renderNode;
}
