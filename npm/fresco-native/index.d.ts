export interface StyleNapi {
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

export interface FlexStyleNapi {
  display?: string;
  position?: string;
  top?: string;
  right?: string;
  bottom?: string;
  left?: string;
  overflow?: string;
  overflowX?: string;
  overflowY?: string;
  flexDirection?: string;
  flexWrap?: string;
  justifyContent?: string;
  alignItems?: string;
  alignSelf?: string;
  alignContent?: string;
  flexGrow?: number;
  flexShrink?: number;
  flexBasis?: string;
  width?: string;
  height?: string;
  minWidth?: string;
  minHeight?: string;
  maxWidth?: string;
  maxHeight?: string;
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

export interface RenderNodeNapi {
  id: number;
  nodeType: string;
  text?: string;
  wrap?: boolean;
  value?: string;
  placeholder?: string;
  focused?: boolean;
  cursor?: number;
  mask?: boolean;
  maskChar?: string;
  style?: FlexStyleNapi;
  appearance?: StyleNapi;
  border?: string;
  children?: number[];
}

export interface ModifiersNapi {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  super: boolean;
  hyper: boolean;
  capsLock: boolean;
  numLock: boolean;
}

export interface InputEventNapi {
  eventType: "key" | "mouse" | "resize" | "focus" | "paste" | string;
  key?: string;
  char?: string;
  keyEventType?: "press" | "repeat" | "release";
  modifiers?: ModifiersNapi;
  button?: string;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  text?: string;
  cursor?: number;
}

export interface ImeStateNapi {
  active: boolean;
  mode: string;
  composing: boolean;
  preedit?: string;
  preeditCursor?: number;
  candidates?: string[];
  selected?: number;
}

export interface TerminalInfoNapi {
  width: number;
  height: number;
  colors: boolean;
  trueColor: boolean;
}

export interface LayoutResultNapi {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export function initTerminal(): void;
export function initTerminalWithMouse(): void;
export function restoreTerminal(): void;
export function getTerminalInfo(): TerminalInfoNapi;
export function clearScreen(): void;
export function flushTerminal(): void;
export function syncTerminalSize(): boolean;

export function initLayout(): void;
export function createLayoutNode(style?: FlexStyleNapi): number;
export function createLayoutLeaf(width: number, height: number, style?: FlexStyleNapi): number;
export function setLayoutRoot(id: number): void;
export function addLayoutChild(parent: number, child: number): void;
export function removeLayoutChild(parent: number, child: number): void;
export function setLayoutStyle(id: number, style: FlexStyleNapi): void;
export function removeLayoutNode(id: number): void;
export function computeLayout(width: number, height: number): void;
export function getLayout(id: number): LayoutResultNapi | undefined;
export function getAllLayouts(): LayoutResultNapi[];
export function clearLayout(): void;

export function renderText(x: number, y: number, text: string, style?: StyleNapi): void;
export function renderBox(
  x: number,
  y: number,
  width: number,
  height: number,
  border?: string,
  style?: StyleNapi,
): void;
export function fillRect(
  x: number,
  y: number,
  width: number,
  height: number,
  char?: string,
  style?: StyleNapi,
): void;
export function clearRect(x: number, y: number, width: number, height: number): void;
export function setCursor(x: number, y: number): void;
export function showCursor(): void;
export function hideCursor(): void;
export function setCursorShape(shape: "block" | "underline" | "bar" | string): void;
export function renderTree(nodes: RenderNodeNapi[]): void;
export function getLastRenderLayouts(): LayoutResultNapi[];

export function pollEvent(timeoutMs: number): InputEventNapi | undefined;
export function pollEventNonBlocking(): InputEventNapi | undefined;
export function readEvent(): InputEventNapi;
export function getImeState(): ImeStateNapi;
export function enableIme(): boolean;
export function disableIme(): boolean;
export function setImeMode(mode: string): boolean;
