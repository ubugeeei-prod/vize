import type { InputEventNapi } from "@vizejs/fresco-native";
import type { KeyEventType, MouseEvent } from "./protocol.js";

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
