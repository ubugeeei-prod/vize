import {
  isCharacterKeyValue,
  parseShortcut,
  readPlatform,
  toShortcutSequence,
} from "./shortcut-parse.ts";
import type {
  ShortcutChord,
  ShortcutFormatOptions,
  ShortcutPlatform,
  ShortcutSequence,
} from "./shortcut-types.ts";

const symbolModifiers: Record<ShortcutPlatform, readonly (readonly [string, string])[]> = {
  // Apple human interface order: Control, Option, Shift, Command.
  apple: [
    ["ctrlKey", "⌃"],
    ["altKey", "⌥"],
    ["shiftKey", "⇧"],
    ["metaKey", "⌘"],
  ],
  standard: [
    ["ctrlKey", "Ctrl"],
    ["altKey", "Alt"],
    ["shiftKey", "Shift"],
    ["metaKey", "Meta"],
  ],
};

const textModifiers: Record<ShortcutPlatform, readonly (readonly [string, string])[]> = {
  apple: [
    ["ctrlKey", "Control"],
    ["altKey", "Option"],
    ["shiftKey", "Shift"],
    ["metaKey", "Command"],
  ],
  standard: symbolModifiers.standard,
};

const symbolKeys = new Map<string, string>([
  ["Enter", "↩"],
  ["Backspace", "⌫"],
  ["Delete", "⌦"],
  ["Escape", "⎋"],
  ["Tab", "⇥"],
  ["CapsLock", "⇪"],
]);

const sharedKeycaps = new Map<string, string>([
  ["ArrowUp", "↑"],
  ["ArrowDown", "↓"],
  ["ArrowLeft", "←"],
  ["ArrowRight", "→"],
  [" ", "Space"],
]);

const textKeys = new Map<string, string>([["Escape", "Esc"]]);

function keycapForKey(key: string, platform: ShortcutPlatform, style: "symbol" | "text"): string {
  const shared = sharedKeycaps.get(key);
  if (shared !== undefined) return shared;
  if (style === "symbol" && platform === "apple") {
    const symbol = symbolKeys.get(key);
    if (symbol !== undefined) return symbol;
  }
  const text = textKeys.get(key);
  if (text !== undefined) return text;
  return isCharacterKeyValue(key) ? key.toUpperCase() : key;
}

function chordKeycaps(
  chord: ShortcutChord,
  platform: ShortcutPlatform,
  style: "symbol" | "text",
): readonly string[] {
  const modifiers = style === "symbol" ? symbolModifiers[platform] : textModifiers[platform];
  const keycaps: string[] = [];
  for (const [property, keycap] of modifiers) {
    if (chord[property as "altKey" | "ctrlKey" | "metaKey" | "shiftKey"]) keycaps.push(keycap);
  }
  keycaps.push(keycapForKey(chord.key, platform, style));
  return Object.freeze(keycaps);
}

function resolveDisplay(options: ShortcutFormatOptions) {
  const platform = readPlatform(options.platform);
  const style = options.style ?? (platform === "apple" ? "symbol" : "text");
  if (style !== "symbol" && style !== "text") {
    throw new TypeError('VIZE_UI_SHORTCUT_FORMAT: style must be "symbol" or "text"');
  }
  return { platform, style } as const;
}

/**
 * Break a shortcut into display keycaps for `<kbd>` rendering.
 *
 * The result carries one array per sequence step; each step lists modifier
 * keycaps in platform order followed by the key keycap. Formatting is
 * deterministic for a given platform, so pass an explicit platform when the
 * output is server-rendered.
 */
export function getShortcutKeycaps(
  shortcut: string | ShortcutSequence,
  options: ShortcutFormatOptions = {},
): readonly (readonly string[])[] {
  const { platform, style } = resolveDisplay(options);
  const sequence =
    typeof shortcut === "string"
      ? parseShortcut(shortcut, { platform })
      : toShortcutSequence(shortcut, platform);
  return Object.freeze(sequence.map((chord) => chordKeycaps(chord, platform, style)));
}

/**
 * Format a shortcut as one display string, e.g. `⇧⌘K` or `Ctrl+Shift+K`.
 *
 * Apple symbol keycaps join without separators; every other combination joins
 * with `+`. Sequence steps are joined with a single space.
 */
export function formatShortcut(
  shortcut: string | ShortcutSequence,
  options: ShortcutFormatOptions = {},
): string {
  const { platform, style } = resolveDisplay(options);
  const joiner = platform === "apple" && style === "symbol" ? "" : "+";
  return getShortcutKeycaps(shortcut, { platform, style })
    .map((keycaps) => keycaps.join(joiner))
    .join(" ");
}
