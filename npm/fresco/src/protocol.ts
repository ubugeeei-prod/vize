/** Public JavaScript protocol shared by Fresco components and the renderer. */

export type FrescoRenderNodeKind = "root" | "box" | "text" | "input";
export type FrescoDimension = number | string;
export type FrescoDisplay = "flex" | "none";
export type FrescoPosition = "absolute" | "relative" | "static";
export type FrescoOverflow = "visible" | "hidden" | "scroll";
export type FrescoFlexDirection = "row" | "column" | "row-reverse" | "column-reverse";
export type FrescoFlexWrap = "nowrap" | "wrap" | "wrap-reverse";
export type FrescoJustifyContent =
  | "flex-start"
  | "flex-end"
  | "start"
  | "end"
  | "center"
  | "space-between"
  | "space-around"
  | "space-evenly";
export type FrescoAlignItems =
  | "flex-start"
  | "flex-end"
  | "start"
  | "end"
  | "center"
  | "stretch"
  | "baseline";
export type FrescoAlignSelf = "auto" | FrescoAlignItems;
export type FrescoAlignContent =
  | "flex-start"
  | "flex-end"
  | "start"
  | "end"
  | "center"
  | "stretch"
  | "space-between"
  | "space-around"
  | "space-evenly";

/** Canonical camelCase style fields. Components accept number or string dimensions. */
export interface FrescoCanonicalStyle<Dimension extends FrescoDimension = FrescoDimension> {
  display?: FrescoDisplay;
  position?: FrescoPosition;
  top?: Dimension;
  right?: Dimension;
  bottom?: Dimension;
  left?: Dimension;
  overflow?: FrescoOverflow;
  overflowX?: FrescoOverflow;
  overflowY?: FrescoOverflow;
  flexDirection?: FrescoFlexDirection;
  flexWrap?: FrescoFlexWrap;
  justifyContent?: FrescoJustifyContent;
  alignItems?: FrescoAlignItems;
  alignSelf?: FrescoAlignSelf;
  alignContent?: FrescoAlignContent;
  flexGrow?: number;
  flexShrink?: number;
  flexBasis?: Dimension;
  width?: Dimension;
  height?: Dimension;
  minWidth?: Dimension;
  minHeight?: Dimension;
  maxWidth?: Dimension;
  maxHeight?: Dimension;
  aspectRatio?: number;
  padding?: number;
  paddingTop?: number;
  paddingRight?: number;
  paddingBottom?: number;
  paddingLeft?: number;
  margin?: number;
  marginTop?: number;
  marginRight?: number;
  marginBottom?: number;
  marginLeft?: number;
  gap?: number;
  columnGap?: number;
  rowGap?: number;
}

/** Compatibility aliases accepted by intrinsic host-node style bags. */
export interface FrescoSnakeStyleAliases<Dimension extends FrescoDimension = FrescoDimension> {
  overflow_x?: FrescoOverflow;
  overflow_y?: FrescoOverflow;
  flex_direction?: FrescoFlexDirection;
  flex_wrap?: FrescoFlexWrap;
  justify_content?: FrescoJustifyContent;
  align_items?: FrescoAlignItems;
  align_self?: FrescoAlignSelf;
  align_content?: FrescoAlignContent;
  flex_grow?: number;
  flex_shrink?: number;
  flex_basis?: Dimension;
  min_width?: Dimension;
  min_height?: Dimension;
  max_width?: Dimension;
  max_height?: Dimension;
  aspect_ratio?: number;
  padding_top?: number;
  padding_right?: number;
  padding_bottom?: number;
  padding_left?: number;
  margin_top?: number;
  margin_right?: number;
  margin_bottom?: number;
  margin_left?: number;
  column_gap?: number;
  row_gap?: number;
}

export type FrescoStyle<Dimension extends FrescoDimension = FrescoDimension> =
  FrescoCanonicalStyle<Dimension> & FrescoSnakeStyleAliases<Dimension>;

/** Canonical style emitted to the native renderer. Dimensions are normalized to strings. */
export type FrescoRenderStyle = FrescoCanonicalStyle<string>;

export interface FrescoAppearance {
  fg?: string;
  bg?: string;
  bold?: boolean;
  dim?: boolean;
  italic?: boolean;
  underline?: boolean;
  inverse?: boolean;
  blink?: boolean;
  hidden?: boolean;
  strikethrough?: boolean;
}

export type FrescoBorderStyle = "none" | "single" | "double" | "rounded" | "heavy" | "dashed";
export type FrescoTextWrapMode =
  | "none"
  | "wrap"
  | "hard"
  | "truncate"
  | "truncate-start"
  | "truncate-middle"
  | "truncate-end";

interface FrescoRenderNodeBase {
  id: number;
  style?: FrescoRenderStyle;
  appearance?: FrescoAppearance;
  border?: FrescoBorderStyle;
  children?: number[];
}

export interface FrescoRootRenderNode extends FrescoRenderNodeBase {
  nodeType: "root";
}

export interface FrescoBoxRenderNode extends FrescoRenderNodeBase {
  nodeType: "box";
}

export interface FrescoTextRenderNode extends FrescoRenderNodeBase {
  nodeType: "text";
  text?: string;
  wrap?: boolean;
  wrapMode?: FrescoTextWrapMode;
}

export interface FrescoInputRenderNode extends FrescoRenderNodeBase {
  nodeType: "input";
  value?: string;
  placeholder?: string;
  focused?: boolean;
  cursor?: number;
  mask?: boolean;
  maskChar?: string;
}

type UnionKeys<Union> = Union extends Union ? keyof Union : never;
type StrictUnionHelper<Member, Union> = Member extends Member
  ? Member & Partial<Record<Exclude<UnionKeys<Union>, keyof Member>, never>>
  : never;
type StrictUnion<Union> = StrictUnionHelper<Union, Union>;

export type FrescoRenderNode = StrictUnion<
  FrescoRootRenderNode | FrescoBoxRenderNode | FrescoTextRenderNode | FrescoInputRenderNode
>;

export type KeyEventType = "press" | "repeat" | "release";

export interface FrescoModifiers {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  super: boolean;
  hyper: boolean;
  capsLock: boolean;
  numLock: boolean;
}

export interface KeyEvent extends FrescoModifiers {
  type: "key";
  key?: string;
  char?: string;
  eventType?: KeyEventType;
}

export interface PasteEvent {
  type: "paste";
  text: string;
}

export interface ResizeEvent {
  type: "resize";
  width: number;
  height: number;
}

export interface MouseEvent {
  type: "mouse";
  button?: string;
  x: number;
  y: number;
}

export interface FocusEvent {
  type: "focus";
  focused: boolean;
}

export interface CompositionEvent {
  type: "compositionstart" | "compositionupdate" | "compositionend";
  text: string;
  cursor: number;
}

export type FrescoInputEvent = StrictUnion<
  KeyEvent | PasteEvent | ResizeEvent | MouseEvent | FocusEvent | CompositionEvent
>;

/** Compatibility alias for the pre-protocol event union name. */
export type InputEvent = FrescoInputEvent;
