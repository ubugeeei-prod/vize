/** Compile-only assertions for the public Fresco protocol entrypoint. */

import type { RenderNodeNapi } from "@vizejs/fresco-native";

import type {
  BoxProps,
  FrescoInputEvent,
  FrescoRenderNode,
  FrescoRenderNodeKind,
  FrescoRenderStyle,
  FrescoStyle,
  InputEvent,
  StaticProps,
  TextInputProps,
  TextProps,
} from "./index.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _FrescoKindIsClosed = Expect<Equal<FrescoRenderNodeKind, "root" | "box" | "text" | "input">>;
type _NativeKindMatchesThePublicProtocol = Expect<
  Equal<RenderNodeNapi["nodeType"], FrescoRenderNodeKind>
>;
type _PublicProtocolFitsTheCurrentNativeBoundary = Expect<
  FrescoRenderNode extends RenderNodeNapi ? true : false
>;
type _InputCompatibilityAliasIsExact = Expect<Equal<InputEvent, FrescoInputEvent>>;

export const authoringStyle: FrescoStyle = {
  width: 40,
  min_width: "25%",
  flexDirection: "column",
  align_items: "center",
  margin_top: 1,
};

export const canonicalRenderStyle: FrescoRenderStyle = {
  width: "40",
  minWidth: "25%",
  flexDirection: "column",
  alignItems: "center",
  marginTop: 1,
};

export const rootNode: FrescoRenderNode = {
  id: -1,
  nodeType: "root",
  style: canonicalRenderStyle,
  children: [1],
};
export const boxNode: FrescoRenderNode = {
  id: 1,
  nodeType: "box",
  border: "rounded",
  appearance: { fg: "cyan", bold: true },
  children: [2, 3],
};
export const textNode: FrescoRenderNode = {
  id: 2,
  nodeType: "text",
  text: "hello",
  wrap: true,
  wrapMode: "wrap",
};
export const inputNode: FrescoRenderNode = {
  id: 3,
  nodeType: "input",
  value: "value",
  placeholder: "type...",
  focused: true,
  cursor: 5,
  mask: true,
  maskChar: "#",
};

// @ts-expect-error - unknown node kinds are outside the closed protocol.
export const unknownNode: FrescoRenderNode = { id: 4, nodeType: "grid" };

// @ts-expect-error - text-only fields cannot cross onto a box variant.
export const mismatchedBoxNode: FrescoRenderNode = { id: 5, nodeType: "box", text: "no" };

// @ts-expect-error - input-only fields cannot cross onto a text variant.
export const mismatchedTextNode: FrescoRenderNode = { id: 6, nodeType: "text", value: "no" };

// @ts-expect-error - style enums reject values the renderer does not understand.
export const invalidStyle: FrescoStyle = { flexDirection: "grid" };

// @ts-expect-error - render payloads use normalized string dimensions.
export const invalidRenderDimension: FrescoRenderStyle = { width: 40 };

export const invalidWrapMode: FrescoRenderNode = {
  id: 7,
  nodeType: "text",
  // @ts-expect-error - wrap modes are a closed native protocol enum.
  wrapMode: "ellipsis",
};

export function exhaustivelyNarrowRenderNode(node: FrescoRenderNode): string {
  switch (node.nodeType) {
    case "root":
      return `root:${node.children?.length ?? 0}`;
    case "box":
      return `box:${node.border ?? "none"}`;
    case "text":
      return `text:${node.text ?? ""}`;
    case "input":
      return `input:${node.value ?? ""}`;
    default: {
      const exhaustive: never = node;
      return exhaustive;
    }
  }
}

export const keyEvent: FrescoInputEvent = {
  type: "key",
  key: "enter",
  ctrl: false,
  alt: false,
  shift: false,
  meta: false,
  super: false,
  hyper: false,
  capsLock: false,
  numLock: false,
  eventType: "press",
};
export const pasteEvent: FrescoInputEvent = { type: "paste", text: "pasted" };
export const resizeEvent: FrescoInputEvent = { type: "resize", width: 80, height: 24 };
export const mouseEvent: FrescoInputEvent = { type: "mouse", button: "left", x: 2, y: 3 };
export const focusEvent: FrescoInputEvent = { type: "focus", focused: true };
export const compositionEvent: FrescoInputEvent = {
  type: "compositionupdate",
  text: "かな",
  cursor: 2,
};

// @ts-expect-error - resize events require both dimensions.
export const missingResizeField: FrescoInputEvent = { type: "resize", width: 80 };

// @ts-expect-error - mouse-only fields cannot cross onto paste events.
export const crossedEventFields: FrescoInputEvent = { type: "paste", text: "x", x: 1, y: 2 };

// @ts-expect-error - native key event phases are closed.
export const invalidKeyPhase: FrescoInputEvent = { ...keyEvent, eventType: "hold" };

export function exhaustivelyNarrowInputEvent(event: FrescoInputEvent): string {
  switch (event.type) {
    case "key":
      return event.key ?? event.char ?? "";
    case "paste":
      return event.text;
    case "resize":
      return `${event.width}x${event.height}`;
    case "mouse":
      return `${event.x},${event.y}`;
    case "focus":
      return String(event.focused);
    case "compositionstart":
    case "compositionupdate":
    case "compositionend":
      return `${event.text}:${event.cursor}`;
    default: {
      const exhaustive: never = event;
      return exhaustive;
    }
  }
}

export const boxKeepsNumericAndStringDimensions: BoxProps = { width: 20, minHeight: "50%" };
export const staticUsesSharedAuthoringStyle: StaticProps<string> = {
  items: ["one"],
  style: { flex_direction: "column", width: 20 },
};
export const textUsesSharedAppearance: TextProps = { content: "hi", blink: true, hidden: false };
export const inputUsesSharedDimension: TextInputProps = { width: "50%" };

// @ts-expect-error - public Static styles are typed rather than arbitrary bags.
export const staticRejectsUntypedStyle: StaticProps = { items: [], style: { mystery: true } };

// @ts-expect-error - component style enums stay connected to the protocol.
export const boxRejectsInvalidProtocolStyle: BoxProps = { alignItems: "middle" };
