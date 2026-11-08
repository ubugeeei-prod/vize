import type { InputEventNapi, ModifiersNapi } from "@vizejs/fresco-native";
import type { CompositionEvent, FrescoInputEvent, KeyEventType, MouseEvent } from "./protocol.js";

/** Accept only key event phases supported by the public Fresco protocol. */
export function normalizeKeyEventType(value: string | undefined): KeyEventType | undefined {
  switch (value) {
    case "press":
    case "repeat":
    case "release":
      return value;
    default:
      return undefined;
  }
}

/** Preserve the generated native mouse contract at the public protocol boundary. */
export function normalizeMouseEvent(event: InputEventNapi): MouseEvent {
  return {
    type: "mouse",
    kind: event.mouseKind ?? undefined,
    button: event.button ?? undefined,
    x: event.x ?? 0,
    y: event.y ?? 0,
    modifiers: event.modifiers
      ? {
          ctrl: event.modifiers.ctrl,
          alt: event.modifiers.alt,
          shift: event.modifiers.shift,
          meta: event.modifiers.meta,
          super: event.modifiers.super,
          hyper: event.modifiers.hyper,
          capsLock: event.modifiers.capsLock,
          numLock: event.modifiers.numLock,
        }
      : undefined,
  };
}

/** Normalize one native input packet into the public Fresco event protocol. */
export function normalizeInputEvent(event: InputEventNapi): FrescoInputEvent | undefined {
  switch (event.eventType) {
    case "resize":
      return { type: "resize", width: event.width ?? 0, height: event.height ?? 0 };
    case "paste":
      return { type: "paste", text: event.text ?? "" };
    case "mouse":
      return normalizeMouseEvent(event);
    case "focus":
      return { type: "focus", focused: event.key === "gained" };
    case "compositionstart":
    case "compositionupdate":
    case "compositionend":
      return {
        type: event.eventType as CompositionEvent["type"],
        text: event.text ?? "",
        cursor: event.cursor ?? 0,
      };
    case "key": {
      const modifiers: Partial<ModifiersNapi> = event.modifiers ?? {};
      return {
        type: "key",
        key: event.key ?? undefined,
        char: event.char ?? undefined,
        ctrl: modifiers.ctrl ?? false,
        alt: modifiers.alt ?? false,
        shift: modifiers.shift ?? false,
        meta: modifiers.meta ?? false,
        super: modifiers.super ?? false,
        hyper: modifiers.hyper ?? false,
        capsLock: modifiers.capsLock ?? false,
        numLock: modifiers.numLock ?? false,
        eventType: normalizeKeyEventType(event.keyEventType),
      };
    }
    default:
      return undefined;
  }
}
