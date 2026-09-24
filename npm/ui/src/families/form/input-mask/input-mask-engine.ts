/**
 * Pure, DOM-free pattern mask engine shared by `useInputMask` and `MaskedInput`.
 *
 * A mask is a string of token characters (slots the user fills) and literal
 * characters (inserted automatically). `\` escapes the next character so a
 * token character can be used as a literal, for example `"\\9"`.
 */

/** One fillable slot kind. */
export interface InputMaskToken {
  /** Single-character test for acceptable input. */
  readonly pattern: RegExp;

  /**
   * Normalizes an accepted character, for example upper-casing.
   *
   * @default undefined
   */
  readonly transform?: (character: string) => string;
}

/** Token table keyed by single mask characters. */
export type InputMaskTokens = Readonly<Record<string, InputMaskToken>>;

let graphemeSegmenter: Intl.Segmenter | undefined;

/** Split text into user-perceived characters so emoji or combined marks stay whole. */
function graphemes(text: string): string[] {
  // Created lazily so importing the module has no side effects.
  graphemeSegmenter ??= new Intl.Segmenter(undefined, { granularity: "grapheme" });
  return Array.from(graphemeSegmenter.segment(text), (part) => part.segment);
}

/** Built-in tokens: `9` digit, `a` ASCII letter, `*` ASCII letter or digit. */
export const INPUT_MASK_DEFAULT_TOKENS = Object.freeze({
  "9": Object.freeze({ pattern: /\d/ }),
  a: Object.freeze({ pattern: /[A-Za-z]/ }),
  "*": Object.freeze({ pattern: /[A-Za-z0-9]/ }),
}) satisfies InputMaskTokens;

/** Token characters available by default. */
export type InputMaskDefaultTokenKey = keyof typeof INPUT_MASK_DEFAULT_TOKENS;

/**
 * Declare custom tokens while keeping their literal keys for inference.
 *
 * @example
 * const hex = defineInputMaskTokens({ H: { pattern: /[0-9a-f]/i, transform: (c) => c.toUpperCase() } });
 */
export function defineInputMaskTokens<const Tokens extends InputMaskTokens>(
  tokens: Tokens,
): Tokens {
  for (const key of Object.keys(tokens)) {
    if (graphemes(key).length !== 1) {
      throw new TypeError(`VIZE_UI_INPUT_MASK_TOKEN: token key "${key}" must be one character`);
    }
  }
  return tokens;
}

/** Options accepted by {@link createInputMask}. */
export interface InputMaskOptions {
  /**
   * Extra or overriding tokens merged over the defaults.
   *
   * @default INPUT_MASK_DEFAULT_TOKENS
   */
  readonly tokens?: InputMaskTokens | undefined;

  /**
   * Character shown in unfilled slots when `lazy` is `false`.
   *
   * @default "_"
   */
  readonly placeholderChar?: string | undefined;

  /**
   * Hide unfilled slots. When `false`, the whole mask is displayed with placeholders.
   *
   * @default true
   */
  readonly lazy?: boolean | undefined;

  /**
   * Append literals that directly follow the last filled slot, so `"(123"`
   * becomes `"(123) "` while typing.
   *
   * @default false
   */
  readonly eager?: boolean | undefined;
}

/** Result of conforming text to a mask. */
export interface InputMaskResult {
  /** Text to display, including literals (and placeholders when not lazy). */
  readonly masked: string;

  /** Only the characters accepted into token slots. */
  readonly raw: string;

  /** Whether every slot is filled. */
  readonly complete: boolean;
}

type MaskSlot =
  | { readonly kind: "token"; readonly token: InputMaskToken }
  | { readonly kind: "literal"; readonly character: string };

/** Compiled mask with conform, caret, and unmask helpers. */
export interface InputMask {
  /** Source mask string. */
  readonly mask: string;

  /** Number of fillable slots. */
  readonly slotCount: number;

  /** Whether every slot accepts only ASCII digits (used for `inputmode`). */
  readonly numeric: boolean;

  /** Conform arbitrary text (typed, pasted, or already masked) to the mask. */
  readonly conform: (text: string) => InputMaskResult;

  /** Conform raw slot characters only; literals in `raw` are never skipped. */
  readonly fromRaw: (raw: string) => InputMaskResult;

  /** Display index just after the `rawCount`-th filled slot. */
  readonly caretForRawCount: (rawCount: number) => number;
}

function parseMask(mask: string, tokens: InputMaskTokens): readonly MaskSlot[] {
  const slots: MaskSlot[] = [];
  const characters = graphemes(mask);
  for (let index = 0; index < characters.length; index += 1) {
    const character = characters[index] ?? "";
    if (character === "\\" && index + 1 < characters.length) {
      index += 1;
      slots.push({ kind: "literal", character: characters[index] ?? "" });
      continue;
    }
    const token = tokens[character];
    slots.push(token === undefined ? { kind: "literal", character } : { kind: "token", token });
  }
  return slots;
}

function accepts(token: InputMaskToken, character: string): boolean {
  token.pattern.lastIndex = 0;
  return token.pattern.test(character);
}

/** Compile a mask string and options into a reusable {@link InputMask}. */
export function createInputMask(mask: string, options: InputMaskOptions = {}): InputMask {
  const tokens = { ...INPUT_MASK_DEFAULT_TOKENS, ...options.tokens };
  const slots = parseMask(mask, tokens);
  const placeholder = graphemes(options.placeholderChar ?? "_")[0] ?? "_";
  const lazy = options.lazy ?? true;
  const eager = options.eager ?? false;
  const slotCount = slots.filter((slot) => slot.kind === "token").length;
  const numeric =
    slotCount > 0 &&
    slots.every((slot) => slot.kind === "literal" || slot.token === INPUT_MASK_DEFAULT_TOKENS["9"]);

  function finish(masked: string, raw: string, slotIndex: number): InputMaskResult {
    let display = masked;
    let index = slotIndex;
    if (eager || !lazy) {
      // Trailing literals directly after the last filled slot.
      while (index < slots.length && slots[index]?.kind === "literal" && raw.length > 0) {
        const slot = slots[index];
        if (slot?.kind === "literal") display += slot.character;
        index += 1;
      }
    }
    if (!lazy) {
      for (; index < slots.length; index += 1) {
        const slot = slots[index];
        if (slot !== undefined) display += slot.kind === "literal" ? slot.character : placeholder;
      }
    }
    return Object.freeze({ masked: display, raw, complete: raw.length === slotCount });
  }

  function run(text: string, fromRawOnly: boolean): InputMaskResult {
    const input = graphemes(text);
    let masked = "";
    let raw = "";
    let inputIndex = 0;
    let slotIndex = 0;
    // Literals are only kept when a filled slot follows them.
    let filledLength = 0;
    let filledSlots = 0;
    while (slotIndex < slots.length) {
      const slot = slots[slotIndex];
      if (slot === undefined) break;
      if (slot.kind === "literal") {
        if (inputIndex >= input.length) break;
        masked += slot.character;
        if (!fromRawOnly && input[inputIndex] === slot.character) inputIndex += 1;
        slotIndex += 1;
        continue;
      }
      while (inputIndex < input.length && !accepts(slot.token, input[inputIndex] ?? "")) {
        inputIndex += 1;
      }
      const character = input[inputIndex];
      if (character === undefined) break;
      const accepted = slot.token.transform?.(character) ?? character;
      masked += accepted;
      raw += accepted;
      inputIndex += 1;
      slotIndex += 1;
      filledLength = masked.length;
      filledSlots = slotIndex;
    }
    return finish(masked.slice(0, filledLength), raw, filledSlots);
  }

  function caretForRawCount(rawCount: number): number {
    let filled = 0;
    let position = 0;
    for (const slot of slots) {
      if (filled >= rawCount) {
        if (slot.kind === "literal" && eager && rawCount > 0) {
          position += 1;
          continue;
        }
        break;
      }
      if (slot.kind === "token") filled += 1;
      position += 1;
    }
    return position;
  }

  return Object.freeze({
    mask,
    slotCount,
    numeric,
    conform: (text: string) => run(text, false),
    fromRaw: (raw: string) => run(raw, true),
    caretForRawCount,
  });
}
