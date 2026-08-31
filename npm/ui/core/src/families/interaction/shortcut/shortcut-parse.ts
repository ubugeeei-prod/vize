import type {
  ShortcutChord,
  ShortcutParseOptions,
  ShortcutPlatform,
  ShortcutSequence,
} from "./shortcut-types.ts";

const parseDiagnostic = "VIZE_UI_SHORTCUT_PATTERN";
const platformDiagnostic = "VIZE_UI_SHORTCUT_PLATFORM";

const modifierTokens = new Map<string, "alt" | "ctrl" | "meta" | "mod" | "shift">([
  ["alt", "alt"],
  ["option", "alt"],
  ["opt", "alt"],
  ["ctrl", "ctrl"],
  ["control", "ctrl"],
  ["meta", "meta"],
  ["cmd", "meta"],
  ["command", "meta"],
  ["win", "meta"],
  ["super", "meta"],
  ["shift", "shift"],
  ["mod", "mod"],
]);

const keyAliases = new Map<string, string>([
  ["esc", "Escape"],
  ["escape", "Escape"],
  ["return", "Enter"],
  ["enter", "Enter"],
  ["space", " "],
  ["spacebar", " "],
  ["plus", "+"],
  ["tab", "Tab"],
  ["backspace", "Backspace"],
  ["delete", "Delete"],
  ["del", "Delete"],
  ["insert", "Insert"],
  ["home", "Home"],
  ["end", "End"],
  ["pageup", "PageUp"],
  ["pagedown", "PageDown"],
  ["up", "ArrowUp"],
  ["arrowup", "ArrowUp"],
  ["down", "ArrowDown"],
  ["arrowdown", "ArrowDown"],
  ["left", "ArrowLeft"],
  ["arrowleft", "ArrowLeft"],
  ["right", "ArrowRight"],
  ["arrowright", "ArrowRight"],
  ["contextmenu", "ContextMenu"],
  ["capslock", "CapsLock"],
  ["numlock", "NumLock"],
  ["scrolllock", "ScrollLock"],
  ["printscreen", "PrintScreen"],
  ["pause", "Pause"],
]);

const platforms = new Set<ShortcutPlatform>(["apple", "standard"]);
const applePlatformPattern = /mac|iphone|ipad|ipod/i;

/**
 * Detect the modifier layout of the current environment.
 *
 * Server rendering has no ambient keyboard, so the detector deterministically
 * reports `standard` there; pass an explicit platform when formatting keycaps
 * into server-rendered markup.
 */
export function detectShortcutPlatform(): ShortcutPlatform {
  const navigatorLike = (globalThis as { navigator?: { platform?: string; userAgent?: string } })
    .navigator;
  if (!navigatorLike) return "standard";
  const surface = `${navigatorLike.platform ?? ""} ${navigatorLike.userAgent ?? ""}`;
  return applePlatformPattern.test(surface) ? "apple" : "standard";
}

export function readPlatform(platform: ShortcutPlatform | undefined): ShortcutPlatform {
  if (platform === undefined) return detectShortcutPlatform();
  if (!platforms.has(platform)) {
    throw new TypeError(`${platformDiagnostic}: platform must be "apple" or "standard"`);
  }
  return platform;
}

/** True when the key value is a single character rather than a named key. */
export function isCharacterKeyValue(value: string): boolean {
  if (value.length === 0) return false;
  return value.length === ((value.codePointAt(0) ?? 0) > 0xffff ? 2 : 1);
}

/** Canonicalize one key token to its `KeyboardEvent.key` value. */
function normalizeKeyToken(token: string): string {
  const alias = keyAliases.get(token.toLowerCase());
  if (alias !== undefined) return alias;
  if (isCharacterKeyValue(token)) return token.toLowerCase();
  if (/^f([1-9]|1\d|2[0-4])$/i.test(token)) return token.toUpperCase();
  return token[0]!.toUpperCase() + token.slice(1);
}

function parseChord(step: string, platform: ShortcutPlatform, pattern: string): ShortcutChord {
  const tokens = step.split("+");
  // A trailing empty token means the literal "+" key, e.g. "Ctrl++".
  if (tokens.length > 1 && tokens.at(-1) === "" && tokens.at(-2) === "") {
    tokens.splice(-2, 2, "plus");
  }
  const chord = { key: "", altKey: false, ctrlKey: false, metaKey: false, shiftKey: false };
  for (const [index, token] of tokens.entries()) {
    if (token === "") {
      throw new TypeError(`${parseDiagnostic}: empty token in "${pattern}"`);
    }
    const modifier = modifierTokens.get(token.toLowerCase());
    if (modifier !== undefined && index < tokens.length - 1) {
      const resolved =
        modifier === "mod" ? (platform === "apple" ? "meta" : "ctrl") : (modifier as string);
      const property = `${resolved}Key` as "altKey" | "ctrlKey" | "metaKey" | "shiftKey";
      if (chord[property]) {
        throw new TypeError(`${parseDiagnostic}: duplicate ${resolved} modifier in "${pattern}"`);
      }
      chord[property] = true;
      continue;
    }
    if (index !== tokens.length - 1) {
      throw new TypeError(`${parseDiagnostic}: "${token}" is not a modifier in "${pattern}"`);
    }
    chord.key = normalizeKeyToken(token);
  }
  if (chord.key === "") {
    throw new TypeError(`${parseDiagnostic}: chord in "${pattern}" is missing a key`);
  }
  return Object.freeze(chord);
}

/**
 * Parse a shortcut pattern into a normalized chord sequence.
 *
 * Chord steps are separated by whitespace and each step joins modifiers and
 * one key with `+`, e.g. `"Mod+K"`, `"Ctrl+Shift+P"`, or `"G D"`. `Mod`
 * resolves to Meta on Apple layouts and Control elsewhere.
 */
export function parseShortcut(
  pattern: string,
  options: ShortcutParseOptions = {},
): ShortcutSequence {
  if (typeof pattern !== "string" || pattern.trim() === "") {
    throw new TypeError(`${parseDiagnostic}: pattern must be a non-empty string`);
  }
  const platform = readPlatform(options.platform);
  const steps = pattern.trim().split(/\s+/);
  return Object.freeze(steps.map((step) => parseChord(step, platform, pattern)));
}

const chordShape = ["key", "altKey", "ctrlKey", "metaKey", "shiftKey"] as const;

/** Validate and freeze a caller-supplied sequence without re-parsing. */
export function toShortcutSequence(
  value: string | ShortcutSequence,
  platform: ShortcutPlatform,
): ShortcutSequence {
  if (typeof value === "string") return parseShortcut(value, { platform });
  if (!Array.isArray(value) || value.length === 0) {
    throw new TypeError(`${parseDiagnostic}: shortcut must be a pattern or a non-empty sequence`);
  }
  return Object.freeze(
    value.map((chord) => {
      if (typeof chord?.key !== "string" || chord.key === "") {
        throw new TypeError(`${parseDiagnostic}: every chord must carry a non-empty key`);
      }
      for (const property of chordShape.slice(1) as readonly (keyof ShortcutChord)[]) {
        if (typeof chord[property] !== "boolean") {
          throw new TypeError(`${parseDiagnostic}: chord ${String(property)} must be a boolean`);
        }
      }
      return Object.freeze({
        key: isCharacterKeyValue(chord.key) ? chord.key.toLowerCase() : chord.key,
        altKey: chord.altKey,
        ctrlKey: chord.ctrlKey,
        metaKey: chord.metaKey,
        shiftKey: chord.shiftKey,
      });
    }),
  );
}

/** Stable identity string used for routing and conflict detection. */
export function serializeShortcut(sequence: ShortcutSequence): string {
  return sequence
    .map((chord) =>
      [
        chord.altKey ? "Alt+" : "",
        chord.ctrlKey ? "Ctrl+" : "",
        chord.metaKey ? "Meta+" : "",
        chord.shiftKey ? "Shift+" : "",
        chord.key === "+" ? "Plus" : chord.key,
      ].join(""),
    )
    .join(" ");
}

const modifierKeys = new Set(["Alt", "AltGraph", "Control", "Meta", "Shift", "OS"]);

/** True when the event is a lone modifier press that can never end a chord. */
export function isModifierOnlyEvent(event: KeyboardEvent): boolean {
  return modifierKeys.has(event.key);
}

/** True when the event satisfies the chord's key and exact modifier state. */
export function matchesShortcutChord(event: KeyboardEvent, chord: ShortcutChord): boolean {
  if (
    event.altKey !== chord.altKey ||
    event.ctrlKey !== chord.ctrlKey ||
    event.metaKey !== chord.metaKey ||
    event.shiftKey !== chord.shiftKey
  ) {
    return false;
  }
  const key = typeof event.key === "string" ? event.key : "";
  return isCharacterKeyValue(chord.key)
    ? key.toLowerCase() === chord.key
    : key.toLowerCase() === chord.key.toLowerCase();
}

/** Build the normalized chord described by one keyboard event. */
export function chordFromEvent(event: KeyboardEvent): ShortcutChord {
  const key = typeof event.key === "string" ? event.key : "";
  return Object.freeze({
    key: isCharacterKeyValue(key) ? key.toLowerCase() : key,
    altKey: event.altKey === true,
    ctrlKey: event.ctrlKey === true,
    metaKey: event.metaKey === true,
    shiftKey: event.shiftKey === true,
  });
}
