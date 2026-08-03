import type { KeyEventType } from "./protocol.js";

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
