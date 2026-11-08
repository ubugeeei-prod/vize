/** Public JavaScript protocol shared by Fresco components and the renderer. */

/** Discriminant shared by JavaScript and native render nodes. */
export type FrescoRenderNodeKind = "root" | "box" | "text" | "input" | "raw";
/** Numeric or CSS-like dimension accepted by component style props. */
export type FrescoDimension = number | string;
/** Display modes supported by the native layout engine. */
export type FrescoDisplay = "flex" | "none";
/** Positioning modes supported by the native layout engine. */
export type FrescoPosition = "absolute" | "relative" | "static";
/** Overflow behavior accepted on either layout axis. */
export type FrescoOverflow = "visible" | "hidden" | "scroll";
/** Main-axis direction accepted by Fresco flex layouts. */
export type FrescoFlexDirection = "row" | "column" | "row-reverse" | "column-reverse";
/** Flex-line wrapping mode accepted by Fresco layouts. */
export type FrescoFlexWrap = "nowrap" | "wrap" | "wrap-reverse";
/** Main-axis distribution accepted by Fresco flex layouts. */
export type FrescoJustifyContent =
  | "flex-start"
  | "flex-end"
  | "start"
  | "end"
  | "center"
  | "space-between"
  | "space-around"
  | "space-evenly";
/** Cross-axis item alignment accepted by Fresco layouts. */
export type FrescoAlignItems =
  | "flex-start"
  | "flex-end"
  | "start"
  | "end"
  | "center"
  | "stretch"
  | "baseline";
/** Per-item cross-axis override accepted by Fresco layouts. */
export type FrescoAlignSelf = "auto" | FrescoAlignItems;
/** Multi-line cross-axis distribution accepted by Fresco layouts. */
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
  /** Signed 32-bit paint and pointer-target order among siblings. Higher values are topmost. @default 0 */
  zIndex?: number;
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
  z_index?: number;
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

/** Complete public style bag with canonical fields and compatibility aliases. */
export type FrescoStyle<Dimension extends FrescoDimension = FrescoDimension> =
  FrescoCanonicalStyle<Dimension> & FrescoSnakeStyleAliases<Dimension>;

/** Canonical style emitted to the native renderer. Dimensions are normalized to strings. */
export type FrescoRenderStyle = FrescoCanonicalStyle<string>;

/** Terminal color and text attributes emitted with a render node. */
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

/** Border glyph family rendered around a box. */
export type FrescoBorderStyle = "none" | "single" | "double" | "rounded" | "heavy" | "dashed";
/** Wrapping or truncation policy for native text nodes. */
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

/** Root node sent to the native renderer. */
export interface FrescoRootRenderNode extends FrescoRenderNodeBase {
  nodeType: "root";
}

/** Layout and appearance-only box node sent to the native renderer. */
export interface FrescoBoxRenderNode extends FrescoRenderNodeBase {
  nodeType: "box";
}

/** Text node sent to the native renderer. */
export interface FrescoTextRenderNode extends FrescoRenderNodeBase {
  nodeType: "text";
  text?: string;
  wrap?: boolean;
  wrapMode?: FrescoTextWrapMode;
}

/** Editable text input node sent to the native renderer. */
export interface FrescoInputRenderNode extends FrescoRenderNodeBase {
  nodeType: "input";
  value?: string;
  placeholder?: string;
  focused?: boolean;
  cursor?: number;
  /** Inclusive grapheme index at the start of the active selection. */
  selectionStart?: number;
  /** Exclusive grapheme index at the end of the active selection. */
  selectionEnd?: number;
  mask?: boolean;
  maskChar?: string;
}

/** Precomposed terminal lines emitted without text wrapping. */
export interface FrescoRawRenderNode extends FrescoRenderNodeBase {
  nodeType: "raw";
  /** Exact terminal rows painted in order inside the computed layout. */
  lines?: string[];
}

type UnionKeys<Union> = Union extends Union ? keyof Union : never;
type StrictUnionHelper<Member, Union> = Member extends Member
  ? Member & Partial<Record<Exclude<UnionKeys<Union>, keyof Member>, never>>
  : never;
type StrictUnion<Union> = StrictUnionHelper<Union, Union>;

/** Strict discriminated union accepted by the native render bridge. */
export type FrescoRenderNode = StrictUnion<
  | FrescoRootRenderNode
  | FrescoBoxRenderNode
  | FrescoTextRenderNode
  | FrescoInputRenderNode
  | FrescoRawRenderNode
>;

/** Lifecycle phase attached to a normalized keyboard event. */
export type KeyEventType = "press" | "repeat" | "release";

/** Modifier state shared by keyboard and mouse events. */
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

/** Normalized keyboard event exposed to components and composables. */
export interface KeyEvent extends FrescoModifiers {
  type: "key";
  key?: string;
  char?: string;
  eventType?: KeyEventType;
}

/** Bracketed-paste input event. */
export interface PasteEvent {
  type: "paste";
  text: string;
}

/** Terminal viewport resize event. */
export interface ResizeEvent {
  type: "resize";
  width: number;
  height: number;
}

/** Mouse transition or scroll direction reported by the terminal. */
export type FrescoMouseEventKind =
  | "down"
  | "up"
  | "drag"
  | "moved"
  | "scroll-up"
  | "scroll-down"
  | "scroll-left"
  | "scroll-right";

/** Normalized mouse event exposed to components and composables. */
export interface MouseEvent {
  type: "mouse";
  kind?: FrescoMouseEventKind;
  button?: string;
  x: number;
  y: number;
  modifiers?: FrescoModifiers;
}

/** Terminal focus-in or focus-out event. */
export interface FocusEvent {
  type: "focus";
  focused: boolean;
}

/** IME composition lifecycle event with text and cursor state. */
export interface CompositionEvent {
  type: "compositionstart" | "compositionupdate" | "compositionend";
  text: string;
  cursor: number;
}

/** Strict union of every public terminal input event. */
export type FrescoInputEvent = StrictUnion<
  KeyEvent | PasteEvent | ResizeEvent | MouseEvent | FocusEvent | CompositionEvent
>;

/** Compatibility alias for the pre-protocol event union name. */
export type InputEvent = FrescoInputEvent;
