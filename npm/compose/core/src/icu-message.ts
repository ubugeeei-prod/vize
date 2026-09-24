/**
 * ICU MessageFormat support: a type-level parameter extractor and a small
 * runtime parser/formatter built on `Intl`.
 *
 * The type-level parser reads message string literals and produces the
 * parameter object a message needs, so a typo in a parameter name or a
 * wrong value type fails to compile. The runtime implements the same
 * grammar subset: simple arguments, `number`, `date`, `time`, `plural`
 * (with `offset:` and `=N` cases), `selectordinal`, `select`, nesting, and
 * ICU apostrophe quoting.
 */

/* -------------------------------------------------------------------------- */
/* Type-level parser                                                          */
/* -------------------------------------------------------------------------- */

/** Whitespace characters skipped around ICU tokens. */
type Whitespace = " " | "\n" | "\t" | "\r";

/** Remove leading and trailing whitespace from a string literal. */
type Trim<Text extends string> = Text extends `${Whitespace}${infer Rest}`
  ? Trim<Rest>
  : Text extends `${infer Rest}${Whitespace}`
    ? Trim<Rest>
    : Text;

/** Characters that start a quoted literal after an apostrophe. */
type QuoteStarter = "{" | "}" | "#" | "|";

/**
 * One parameter occurrence found while scanning: `[name, value type,
 * whether the occurrence carries a format type]`.
 */
type Occurrence = readonly [name: string, value: unknown, typed: boolean];

/** Value type required by an argument's format type. */
type ArgumentValue<Kind extends string> = Kind extends
  | "number"
  | "spellout"
  | "ordinal"
  | "duration"
  ? number
  : Kind extends "date" | "time"
    ? Date | number
    : string;

/** Skip a quoted literal (the text after `'`) up to its closing apostrophe. */
type SkipQuoted<Text extends string> = Text extends `${string}'${infer Rest}` ? Rest : "";

/**
 * Scan message text, collecting argument occurrences. Plain text is skipped
 * in one step up to the next `{` unless it contains an apostrophe, in which
 * case quoting is handled character by character.
 */
type Scan<
  Text extends string,
  Found extends Occurrence,
> = Text extends `${infer Plain}{${infer Rest}`
  ? Plain extends `${string}'${string}`
    ? ScanChars<Text, Found>
    : ScanArgument<Rest, Found>
  : Found;

/** Character-level scan used around apostrophes. */
type ScanChars<Text extends string, Found extends Occurrence> = Text extends `''${infer Rest}`
  ? ScanChars<Rest, Found>
  : Text extends `'${infer Next}${infer Rest}`
    ? Next extends QuoteStarter
      ? Scan<SkipQuoted<Rest>, Found>
      : ScanChars<`${Next}${Rest}`, Found>
    : Text extends `{${infer Rest}`
      ? ScanArgument<Rest, Found>
      : Text extends `${string}${infer Rest}`
        ? Text extends ""
          ? Found
          : ScanChars<Rest, Found>
        : Found;

/** Parse the argument that starts after a `{`. */
type ScanArgument<
  Text extends string,
  Found extends Occurrence,
> = Text extends `${infer Head}}${infer AfterSimple}`
  ? Head extends `${string},${string}`
    ? Text extends `${infer Name},${infer AfterName}`
      ? ScanFormat<Trim<Name>, AfterName, Found>
      : Found
    : Scan<AfterSimple, Found | readonly [Trim<Head>, string, false]>
  : Found;

/** Parse `type[, style]}` or `type, cases}` after an argument name. */
type ScanFormat<
  Name extends string,
  Text extends string,
  Found extends Occurrence,
> = Text extends `${infer Kind},${infer AfterKind}`
  ? Kind extends `${string}}${string}`
    ? ScanSimpleFormat<Name, Text, Found>
    : Trim<Kind> extends "plural" | "selectordinal"
      ? ScanCases<SkipOffset<AfterKind>, Name, number, never, Found>
      : Trim<Kind> extends "select"
        ? ScanCases<AfterKind, Name, never, never, Found>
        : AfterKind extends `${string}}${infer Rest}`
          ? Scan<Rest, Found | readonly [Name, ArgumentValue<Trim<Kind>>, true]>
          : Found
  : ScanSimpleFormat<Name, Text, Found>;

/** Parse `type}` (a format type without style). */
type ScanSimpleFormat<
  Name extends string,
  Text extends string,
  Found extends Occurrence,
> = Text extends `${infer Kind}}${infer Rest}`
  ? Scan<Rest, Found | readonly [Name, ArgumentValue<Trim<Kind>>, true]>
  : Found;

/** Drop a plural `offset:N` prefix. */
type SkipOffset<Text extends string> =
  Trim<Text> extends `offset:${infer Rest}`
    ? Rest extends `${infer _Digits}${Whitespace}${infer Cases}`
      ? Cases
      : Rest
    : Text;

/**
 * Parse `key {content} key {content} ... }`. `Value` is the fixed argument
 * type (`number` for plural kinds); for `select` it stays `never` and the
 * case keys are collected in `Keys` instead.
 */
type ScanCases<
  Text extends string,
  Name extends string,
  Value,
  Keys extends string,
  Found extends Occurrence,
> =
  Trim<Text> extends `}${infer Rest}`
    ? Scan<Rest, Found | readonly [Name, [Value] extends [never] ? Keys : Value, true]>
    : Trim<Text> extends `${infer Key}{${infer Body}`
      ? TakeBalanced<Body, "", []> extends [infer Content extends string, infer Rest extends string]
        ? ScanCases<Rest, Name, Value, Keys | Trim<Key>, Found | Scan<Content, never>>
        : Found
      : Found;

/**
 * Split `content}rest` at the `}` that closes the current case, honoring
 * nested braces and apostrophe quoting. Returns `[content, rest]`.
 */
type TakeBalanced<
  Text extends string,
  Taken extends string,
  Depth extends readonly unknown[],
> = Text extends `''${infer Rest}`
  ? TakeBalanced<Rest, `${Taken}''`, Depth>
  : Text extends `'${infer Next}${infer Rest}`
    ? Next extends QuoteStarter
      ? Rest extends `${infer Quoted}'${infer After}`
        ? TakeBalanced<After, `${Taken}'${Next}${Quoted}'`, Depth>
        : [`${Taken}'${Next}${Rest}`, ""]
      : TakeBalanced<`${Next}${Rest}`, `${Taken}'`, Depth>
    : Text extends `{${infer Rest}`
      ? TakeBalanced<Rest, `${Taken}{`, [...Depth, unknown]>
      : Text extends `}${infer Rest}`
        ? Depth extends readonly [unknown, ...infer Outer]
          ? TakeBalanced<Rest, `${Taken}}`, Outer>
          : [Taken, Rest]
        : Text extends `${infer Char}${infer Rest}`
          ? TakeBalanced<Rest, `${Taken}${Char}`, Depth>
          : [Taken, ""];

/** Flatten an intersection into a single object type. */
type Simplify<Value> = { [Key in keyof Value]: Value[Key] } & {};

/** Intersect the members of a union. */
type UnionToIntersection<Union> = (Union extends unknown ? (value: Union) => void : never) extends (
  value: infer Intersection,
) => void
  ? Intersection
  : never;

/**
 * Value type of one parameter: typed occurrences win over plain `{name}`
 * occurrences and are intersected with each other.
 */
type ResolveOccurrences<Entries extends Occurrence> = [
  Extract<Entries, readonly [string, unknown, true]>,
] extends [never]
  ? string
  : UnionToIntersection<BoxTypedValue<Entries>> extends infer Merged
    ? Merged extends { readonly value: unknown }
      ? Merged["value"]
      : never
    : never;

/** Box the value of each typed occurrence (distributive over `Entry`). */
type BoxTypedValue<Entry extends Occurrence> = Entry extends readonly [string, infer Value, true]
  ? { readonly value: Value }
  : never;

/**
 * Every argument occurrence of a message literal as a union of
 * `[name, value type, typed]` tuples. Exposed so that several messages (for
 * example the translations of one key) can be merged with the same
 * "typed occurrence wins" rule through {@link ResolveMessageArguments}.
 */
export type MessageArguments<Message extends string> = Scan<Message, never>;

/**
 * Turn a union of {@link MessageArguments} into a parameter object. Typed
 * occurrences win over plain `{name}` ones; conflicting typed occurrences
 * intersect (and become `never` when incompatible).
 */
export type ResolveMessageArguments<Entries extends Occurrence> = Simplify<CollectValues<Entries>>;

/** One argument occurrence: `[name, value type, whether it is typed]`. */
export type MessageArgument = Occurrence;

/** Distribute a union of occurrences into one value type per name. */
type CollectValues<Entries extends Occurrence> = {
  [Name in Entries[0]]: ResolveOccurrences<Extract<Entries, readonly [Name, unknown, boolean]>>;
};

/**
 * Parameters a single ICU message needs, extracted from its string literal.
 *
 * - `{name}` → `string`
 * - `{n, number}` (also `spellout`, `ordinal`, `duration`) → `number`
 * - `{d, date}` / `{d, time}` → `Date | number`
 * - `{n, plural, …}` / `{n, selectordinal, …}` → `number`
 * - `{g, select, a {…} b {…} other {…}}` → `"a" | "b" | "other"`
 *
 * Nested arguments inside plural/select cases are collected as well. When
 * a name occurs both plainly and with a type, the typed occurrence wins.
 * A non-literal `string` yields `Record<string, unknown>`.
 *
 * @example
 * ```ts
 * type P = MessageParams<"Hi {name}, {count, plural, one {# item} other {# items}}">;
 * //   ^? { name: string; count: number }
 * ```
 */
export type MessageParams<Message extends string> = string extends Message
  ? Record<string, unknown>
  : Simplify<CollectValues<Scan<Message, never>>>;

/** Whether a message literal takes no parameters. */
export type HasNoParams<Message extends string> = keyof MessageParams<Message> extends never
  ? true
  : false;

/* -------------------------------------------------------------------------- */
/* Runtime parser                                                             */
/* -------------------------------------------------------------------------- */

/** Node of a parsed ICU message. */
export type MessageNode =
  | { readonly kind: "text"; readonly value: string }
  | { readonly kind: "pound" }
  | { readonly kind: "argument"; readonly name: string }
  | {
      readonly kind: "number" | "date" | "time";
      readonly name: string;
      readonly style: string | undefined;
    }
  | {
      readonly kind: "plural" | "selectordinal";
      readonly name: string;
      readonly offset: number;
      readonly cases: Readonly<Record<string, readonly MessageNode[]>>;
    }
  | {
      readonly kind: "select";
      readonly name: string;
      readonly cases: Readonly<Record<string, readonly MessageNode[]>>;
    };

/** Error code carried by {@link MessageSyntaxError}. */
export type MessageSyntaxErrorCode = "VIZE_COMPOSE_ICU_SYNTAX";

/** Thrown by {@link parseMessage} for malformed ICU messages. */
export interface MessageSyntaxError extends SyntaxError {
  /** Stable machine-readable code. */
  readonly code: MessageSyntaxErrorCode;
  /** Character offset of the problem. */
  readonly offset: number;
}

function syntaxError(message: string, offset: number): MessageSyntaxError {
  return Object.assign(
    new SyntaxError(`[VIZE_COMPOSE_ICU_SYNTAX] ${message} at offset ${String(offset)}`),
    { code: "VIZE_COMPOSE_ICU_SYNTAX" as const, offset },
  );
}

const quoteStarters = new Set(["{", "}", "#", "|"]);

/**
 * Parse an ICU MessageFormat string into an AST.
 *
 * Supports simple arguments, `number`/`date`/`time` with an optional style,
 * `plural` and `selectordinal` (with `offset:` and `=N` cases), `select`,
 * nesting, `#` inside plural cases, and apostrophe quoting (`''` is a
 * literal apostrophe; `'{…}'` is literal text). Pure and deterministic.
 *
 * @example
 * ```ts
 * parseMessage("{n, plural, one {# item} other {# items}}");
 * ```
 *
 * @param message ICU message source.
 * @throws {@link MessageSyntaxError} tagged `VIZE_COMPOSE_ICU_SYNTAX` for
 * malformed messages, including a plural or select without an `other` case.
 * @returns The parsed nodes.
 */
export function parseMessage(message: string): MessageNode[] {
  let position = 0;

  const skipSpace = (): void => {
    while (position < message.length && /\s/.test(message.charAt(position))) position += 1;
  };

  const readIdentifier = (stops: string): string => {
    const start = position;
    while (position < message.length && !stops.includes(message.charAt(position))) position += 1;
    return message.slice(start, position).trim();
  };

  const expect = (char: string): void => {
    if (message.charAt(position) !== char) {
      throw syntaxError(`expected "${char}"`, position);
    }
    position += 1;
  };

  const parseNodes = (inPlural: boolean, nested: boolean): MessageNode[] => {
    const nodes: MessageNode[] = [];
    let text = "";
    const flush = (): void => {
      if (text !== "") nodes.push({ kind: "text", value: text });
      text = "";
    };
    while (position < message.length) {
      const char = message.charAt(position);
      if (char === "'") {
        const next = message.charAt(position + 1);
        if (next === "'") {
          text += "'";
          position += 2;
        } else if (quoteStarters.has(next)) {
          const end = message.indexOf("'", position + 1);
          text += message.slice(position + 1, end === -1 ? message.length : end);
          position = end === -1 ? message.length : end + 1;
        } else {
          text += "'";
          position += 1;
        }
      } else if (char === "{") {
        flush();
        position += 1;
        nodes.push(parseArgument());
      } else if (char === "}") {
        if (!nested) throw syntaxError('unexpected "}"', position);
        break;
      } else if (char === "#" && inPlural) {
        flush();
        nodes.push({ kind: "pound" });
        position += 1;
      } else {
        text += char;
        position += 1;
      }
    }
    flush();
    return nodes;
  };

  const parseCases = (inPlural: boolean): Record<string, MessageNode[]> => {
    const cases: Record<string, MessageNode[]> = {};
    for (;;) {
      skipSpace();
      if (message.charAt(position) === "}") break;
      const key = readIdentifier("{}");
      if (key === "" || /\s/.test(key)) throw syntaxError("invalid case key", position);
      expect("{");
      cases[key] = parseNodes(inPlural, true);
      expect("}");
    }
    if (cases["other"] === undefined) throw syntaxError('missing "other" case', position);
    return cases;
  };

  const parseArgument = (): MessageNode => {
    skipSpace();
    const name = readIdentifier(",}");
    if (name === "") throw syntaxError("empty argument name", position);
    if (message.charAt(position) === "}") {
      position += 1;
      return { kind: "argument", name };
    }
    expect(",");
    const kind = readIdentifier(",}");
    let node: MessageNode;
    if (kind === "plural" || kind === "selectordinal") {
      expect(",");
      skipSpace();
      let offset = 0;
      const match = /^offset:\s*(\d+)/.exec(message.slice(position));
      if (match?.[1] !== undefined) {
        offset = Number(match[1]);
        position += match[0].length;
      }
      node = { kind, name, offset, cases: parseCases(true) };
    } else if (kind === "select") {
      expect(",");
      node = { kind, name, cases: parseCases(false) };
    } else if (kind === "number" || kind === "date" || kind === "time") {
      let style: string | undefined;
      if (message.charAt(position) === ",") {
        position += 1;
        style = readIdentifier("}");
      }
      node = { kind, name, style };
    } else {
      throw syntaxError(`unsupported argument type "${kind}"`, position);
    }
    skipSpace();
    expect("}");
    return node;
  };

  const nodes = parseNodes(false, false);
  return nodes;
}

/* -------------------------------------------------------------------------- */
/* Runtime formatter                                                          */
/* -------------------------------------------------------------------------- */

/** Options for {@link formatMessage} and {@link createMessageFormatter}. */
export interface FormatMessageOptions {
  /**
   * IANA time zone for `date` and `time` arguments. Fixed by default so
   * server and client render identically.
   *
   * @default "UTC"
   */
  readonly timeZone?: string;

  /**
   * Named `number` styles usable as `{n, number, name}`.
   *
   * @default {}
   */
  readonly numberFormats?: Readonly<Record<string, Intl.NumberFormatOptions>>;

  /**
   * Named `date`/`time` styles usable as `{d, date, name}`.
   *
   * @default {}
   */
  readonly dateTimeFormats?: Readonly<Record<string, Intl.DateTimeFormatOptions>>;
}

/** Formats parsed messages for one locale, caching `Intl` objects. */
export interface MessageFormatter {
  /** Locale the formatter renders in. */
  readonly locale: string;

  /**
   * Render a message source or pre-parsed nodes.
   *
   * @returns The formatted string.
   */
  readonly format: (
    message: string | readonly MessageNode[],
    params?: Readonly<Record<string, unknown>>,
  ) => string;
}

const dateStyles = new Set(["short", "medium", "long", "full"]);

function numberOptions(
  style: string | undefined,
  named: Readonly<Record<string, Intl.NumberFormatOptions>>,
): Intl.NumberFormatOptions {
  if (style === undefined) return {};
  const custom = named[style];
  if (custom !== undefined) return custom;
  if (style === "integer") return { maximumFractionDigits: 0 };
  if (style === "percent") return { style: "percent" };
  if (!style.startsWith("::")) return {};
  const options: Intl.NumberFormatOptions = {};
  for (const token of style.slice(2).trim().split(/\s+/)) {
    if (token.startsWith("currency/")) {
      Object.assign(options, { style: "currency", currency: token.slice(9) });
    } else if (token === "percent") {
      Object.assign(options, { style: "percent" });
    } else if (token === "compact-short" || token === "K") {
      Object.assign(options, { notation: "compact", compactDisplay: "short" });
    } else if (token === "compact-long" || token === "KK") {
      Object.assign(options, { notation: "compact", compactDisplay: "long" });
    } else if (token === "group-off" || token === ",_") {
      Object.assign(options, { useGrouping: false });
    } else if (/^\.0*#*$/.test(token)) {
      const minimum = token.slice(1).replaceAll("#", "").length;
      Object.assign(options, {
        minimumFractionDigits: minimum,
        maximumFractionDigits: token.length - 1,
      });
    }
  }
  return options;
}

function dateTimeOptions(
  kind: "date" | "time",
  style: string | undefined,
  named: Readonly<Record<string, Intl.DateTimeFormatOptions>>,
): Intl.DateTimeFormatOptions {
  const custom = style === undefined ? undefined : named[style];
  if (custom !== undefined) return custom;
  const resolved = style !== undefined && dateStyles.has(style) ? style : "medium";
  const value =
    resolved === "short" || resolved === "long" || resolved === "full" ? resolved : "medium";
  return kind === "date" ? { dateStyle: value } : { timeStyle: value };
}

/** Stringify an argument value; plain objects use JSON rather than `[object Object]`. */
function stringify(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "bigint" || typeof value === "boolean" || typeof value === "symbol") {
    return value.toString();
  }
  return typeof value === "function" ? "[function]" : JSON.stringify(value);
}

/**
 * Create a caching formatter for one locale.
 *
 * Parsed messages and `Intl` formatters are cached inside the returned
 * object only (no module state), so one formatter per request or per
 * i18n instance keeps server rendering isolated.
 *
 * @example
 * ```ts
 * const formatter = createMessageFormatter("en");
 * formatter.format("{n, plural, one {# file} other {# files}}", { n: 3 }); // "3 files"
 * ```
 *
 * @param locale BCP 47 locale.
 * @param options Time zone and named formats.
 * @default options {}
 * @returns The formatter.
 */
export function createMessageFormatter(
  locale: string,
  options: FormatMessageOptions = {},
): MessageFormatter {
  const timeZone = options.timeZone ?? "UTC";
  const numberFormats = options.numberFormats ?? {};
  const dateTimeFormats = options.dateTimeFormats ?? {};
  const parsed = new Map<string, readonly MessageNode[]>();
  const numbers = new Map<string, Intl.NumberFormat>();
  const dates = new Map<string, Intl.DateTimeFormat>();
  const plurals = new Map<string, Intl.PluralRules>();

  const number = (style: string | undefined): Intl.NumberFormat => {
    const key = style ?? "";
    let formatter = numbers.get(key);
    if (formatter === undefined) {
      formatter = new Intl.NumberFormat(locale, numberOptions(style, numberFormats));
      numbers.set(key, formatter);
    }
    return formatter;
  };
  const dateTime = (kind: "date" | "time", style: string | undefined): Intl.DateTimeFormat => {
    const key = `${kind}:${style ?? ""}`;
    let formatter = dates.get(key);
    if (formatter === undefined) {
      formatter = new Intl.DateTimeFormat(locale, {
        ...dateTimeOptions(kind, style, dateTimeFormats),
        timeZone,
      });
      dates.set(key, formatter);
    }
    return formatter;
  };
  const pluralRules = (type: Intl.PluralRuleType): Intl.PluralRules => {
    let rules = plurals.get(type);
    if (rules === undefined) {
      rules = new Intl.PluralRules(locale, { type });
      plurals.set(type, rules);
    }
    return rules;
  };

  const toNumber = (value: unknown): number => (typeof value === "number" ? value : Number(value));
  const toDate = (value: unknown): Date =>
    value instanceof Date ? value : new Date(typeof value === "number" ? value : String(value));

  const render = (
    nodes: readonly MessageNode[],
    params: Readonly<Record<string, unknown>>,
    pound: number | undefined,
  ): string => {
    let output = "";
    for (const node of nodes) {
      switch (node.kind) {
        case "text":
          output += node.value;
          break;
        case "pound":
          output += pound === undefined ? "#" : number(undefined).format(pound);
          break;
        case "argument": {
          const value = params[node.name];
          output +=
            value instanceof Date
              ? dateTime("date", undefined).format(value)
              : typeof value === "number"
                ? number(undefined).format(value)
                : value === undefined || value === null
                  ? `{${node.name}}`
                  : stringify(value);
          break;
        }
        case "number":
          output += number(node.style).format(toNumber(params[node.name]));
          break;
        case "date":
        case "time":
          output += dateTime(node.kind, node.style).format(toDate(params[node.name]));
          break;
        case "plural":
        case "selectordinal": {
          const value = toNumber(params[node.name]);
          const exact = node.cases[`=${String(value)}`];
          const shifted = value - node.offset;
          const category = pluralRules(node.kind === "plural" ? "cardinal" : "ordinal").select(
            shifted,
          );
          const branch = exact ?? node.cases[category] ?? node.cases["other"] ?? [];
          output += render(branch, params, shifted);
          break;
        }
        case "select": {
          const value = String(params[node.name]);
          const branch = node.cases[value] ?? node.cases["other"] ?? [];
          output += render(branch, params, pound);
          break;
        }
      }
    }
    return output;
  };

  return {
    locale,
    format: (message, params = {}) => {
      let nodes: readonly MessageNode[];
      if (typeof message === "string") {
        const cached = parsed.get(message);
        nodes = cached ?? parseMessage(message);
        if (cached === undefined) parsed.set(message, nodes);
      } else {
        nodes = message;
      }
      return render(nodes, params, undefined);
    },
  };
}

/**
 * Format one ICU message with typed parameters.
 *
 * Parameters are checked against the message literal at compile time via
 * {@link MessageParams}. For repeated formatting prefer
 * {@link createMessageFormatter}, which caches parsing and `Intl` objects.
 * Deterministic: `date`/`time` arguments use `timeZone` (UTC by default).
 *
 * @example
 * ```ts
 * formatMessage("en", "Hello {name}!", { name: "Ada" }); // "Hello Ada!"
 * ```
 *
 * @param locale BCP 47 locale.
 * @param message ICU message.
 * @param params Values for the message's arguments.
 * @param options Time zone and named formats.
 * @default options {}
 * @throws {@link MessageSyntaxError} for malformed messages.
 * @returns The formatted string.
 */
export function formatMessage<const Message extends string>(
  locale: string,
  message: Message,
  params: MessageParams<Message>,
  options: FormatMessageOptions = {},
): string {
  return createMessageFormatter(locale, options).format(message, params);
}
