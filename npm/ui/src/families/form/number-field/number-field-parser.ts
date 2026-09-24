/**
 * Locale-aware number formatting and parsing built only on `Intl.NumberFormat`.
 *
 * The parser learns every locale-specific symbol (group and decimal separators,
 * minus/plus signs, currency, percent and unit affixes, and native numerals)
 * from `formatToParts`, so it round-trips whatever the formatter prints for the
 * same locale and options without shipping locale data.
 */

/** Intl options accepted by NumberField formatting and parsing. */
export type NumberFieldFormatOptions = Intl.NumberFormatOptions;

/** Locale-aware formatter and parser pair for one locale and option set. */
export interface NumberFieldParser {
  /** Resolved BCP 47 locale used by the formatter. */
  readonly locale: string;

  /** Resolved Intl options, including the numbering system and fraction digits. */
  readonly resolvedOptions: Intl.ResolvedNumberFormatOptions;

  /** Format a finite number for display. */
  readonly format: (value: number) => string;

  /**
   * Parse displayed or typed text into a number.
   *
   * Returns `null` for empty text and `Number.NaN` for text that is not a number.
   */
  readonly parse: (text: string) => number | null;

  /**
   * Whether text is a valid prefix of a number the user may still be typing,
   * for example `""`, `"-"`, or `"1,"` in German.
   */
  readonly isPartial: (text: string, options?: NumberFieldPartialOptions) => boolean;
}

/** Constraints applied while validating partially typed text. */
export interface NumberFieldPartialOptions {
  /**
   * Whether a leading minus sign is acceptable.
   *
   * @default true
   */
  readonly allowNegative?: boolean;
}

const asciiDigits = "0123456789";
const bidiMarks = /[؜‎‏‪-‮⁦-⁩]/g;
const spaces = /[\s  ]/g;
const minusSigns = ["-", "−", "‒", "–", "—", "﹣", "－"];
const plusSigns = ["+", "﹢", "＋"];
const partialPattern = /^[+-]?\d*(?:\.\d*)?$/;
const completePattern = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)$/;
const parserCache = new Map<string, NumberFieldParser>();
const maximumCachedParsers = 64;

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function stripInvisible(value: string): string {
  return value.replace(bidiMarks, "");
}

function cacheKey(locale: string, options: NumberFieldFormatOptions | undefined): string {
  if (options === undefined) return locale;
  const entries = Object.entries(options)
    .filter(([, value]) => value !== undefined)
    .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));
  return `${locale}|${JSON.stringify(entries)}`;
}

function partsOf(formatter: Intl.NumberFormat, value: number): Intl.NumberFormatPart[] {
  return formatter.formatToParts(value);
}

function firstPart(
  parts: readonly Intl.NumberFormatPart[],
  type: Intl.NumberFormatPartTypes,
): string | undefined {
  return parts.find((part) => part.type === type)?.value;
}

interface LocaleSymbols {
  readonly decimal: string;
  readonly group: string | undefined;
  readonly minus: readonly string[];
  readonly plus: readonly string[];
  readonly affixes: readonly string[];
  readonly numerals: ReadonlyMap<string, string>;
  readonly percent: boolean;
  readonly accounting: boolean;
}

function collectSymbols(
  locale: string,
  options: NumberFieldFormatOptions,
  resolved: Intl.ResolvedNumberFormatOptions,
): LocaleSymbols {
  const numberingSystem = resolved.numberingSystem;
  const separators = new Intl.NumberFormat(locale, {
    numberingSystem,
    useGrouping: true,
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  });
  const separatorParts = partsOf(separators, -11_111.1);
  const decimal = stripInvisible(firstPart(separatorParts, "decimal") ?? ".");
  const rawGroup = firstPart(separatorParts, "group");
  const group = rawGroup === undefined ? undefined : stripInvisible(rawGroup);
  const localizedMinus = stripInvisible(firstPart(separatorParts, "minusSign") ?? "-");
  const signed = new Intl.NumberFormat(locale, { numberingSystem, signDisplay: "always" });
  const localizedPlus = stripInvisible(firstPart(partsOf(signed, 1), "plusSign") ?? "+");

  const digitFormatter = new Intl.NumberFormat(locale, { numberingSystem, useGrouping: false });
  const numerals = new Map<string, string>();
  for (const digit of asciiDigits) {
    numerals.set(digitFormatter.format(Number(digit)), digit);
  }

  const display = new Intl.NumberFormat(locale, options);
  const affixes = new Set<string>();
  for (const sample of [-1, 1, 0, -1234.5, 1234.5]) {
    for (const part of partsOf(display, sample)) {
      if (
        part.type === "currency" ||
        part.type === "unit" ||
        part.type === "percentSign" ||
        part.type === "literal" ||
        part.type === "compact"
      ) {
        const affix = stripInvisible(part.value).replace(spaces, "");
        if (affix.length > 0 && affix !== "(" && affix !== ")") affixes.add(affix);
      }
    }
  }

  return {
    decimal,
    group,
    minus: [...new Set([localizedMinus, ...minusSigns])],
    plus: [...new Set([localizedPlus, ...plusSigns])],
    // Longest first so `US$` is removed before `$`.
    affixes: [...affixes].sort((left, right) => right.length - left.length),
    numerals,
    percent: resolved.style === "percent",
    accounting: options.currencySign === "accounting",
  };
}

function roundTo(value: number, fractionDigits: number): number {
  const digits = Math.min(Math.max(Math.trunc(fractionDigits), 0), 20);
  return Number(value.toFixed(digits));
}

/**
 * Create (or reuse) a locale-aware NumberField formatter and parser.
 *
 * Parsers are cached per locale and option set, so repeated calls during
 * render are cheap and deterministic across server and client.
 */
export function createNumberFieldParser(
  locale: string,
  options: NumberFieldFormatOptions = {},
): NumberFieldParser {
  const key = cacheKey(locale, options);
  const cached = parserCache.get(key);
  if (cached !== undefined) return cached;

  const formatter = new Intl.NumberFormat(locale, options);
  const resolvedOptions = formatter.resolvedOptions();
  const symbols = collectSymbols(resolvedOptions.locale, options, resolvedOptions);
  const allowsFraction = resolvedOptions.maximumFractionDigits !== 0 || symbols.percent;
  const affixPattern =
    symbols.affixes.length === 0
      ? undefined
      : new RegExp(symbols.affixes.map(escapeRegExp).join("|"), "g");
  const groupPattern =
    symbols.group === undefined || symbols.group.replace(spaces, "").length === 0
      ? undefined
      : new RegExp(escapeRegExp(symbols.group), "g");

  /** Normalize localized text into an ASCII `[+-]digits[.digits]` candidate. */
  function normalize(text: string): { readonly candidate: string; readonly negative: boolean } {
    let working = stripInvisible(text).replace(spaces, "");
    let negative = false;
    if (symbols.accounting && working.startsWith("(") && working.endsWith(")")) {
      negative = true;
      working = working.slice(1, -1);
    }
    if (affixPattern !== undefined) working = working.replace(affixPattern, "");
    if (groupPattern !== undefined) working = working.replace(groupPattern, "");

    let candidate = "";
    for (const character of working) {
      const numeral = symbols.numerals.get(character);
      if (numeral !== undefined) candidate += numeral;
      else if (asciiDigits.includes(character)) candidate += character;
      else if (character === symbols.decimal) candidate += ".";
      else if (symbols.minus.includes(character)) candidate += "-";
      else if (symbols.plus.includes(character)) candidate += "+";
      else candidate += character;
    }
    return { candidate, negative };
  }

  const parser: NumberFieldParser = Object.freeze({
    locale: resolvedOptions.locale,
    resolvedOptions,
    format: (value: number) => (Number.isFinite(value) ? formatter.format(value) : ""),
    parse(text: string): number | null {
      const { candidate, negative } = normalize(text);
      if (candidate.length === 0) return null;
      if (!completePattern.test(candidate)) return Number.NaN;
      if (!allowsFraction && candidate.includes(".")) return Number.NaN;
      let value = Number(candidate);
      if (!Number.isFinite(value)) return Number.NaN;
      if (negative) value = -Math.abs(value);
      if (symbols.percent) {
        const fraction = candidate.split(".")[1]?.length ?? 0;
        value = roundTo(value / 100, fraction + 2);
      }
      return Object.is(value, -0) ? 0 : value;
    },
    isPartial(text: string, partialOptions: NumberFieldPartialOptions = {}): boolean {
      const { candidate, negative } = normalize(text);
      if (!partialPattern.test(candidate)) return false;
      if (!allowsFraction && candidate.includes(".")) return false;
      if (partialOptions.allowNegative === false && (negative || candidate.startsWith("-"))) {
        return false;
      }
      return true;
    },
  });

  if (parserCache.size >= maximumCachedParsers) {
    const oldest = parserCache.keys().next();
    if (oldest.done !== true) parserCache.delete(oldest.value);
  }
  parserCache.set(key, parser);
  return parser;
}
