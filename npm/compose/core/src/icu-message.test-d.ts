/** Compile-only assertions for the type-level ICU MessageFormat parser. */

import { formatMessage } from "./icu-message.ts";
import type { HasNoParams, MessageArguments, MessageParams } from "./icu-message.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

// Simple and plural arguments.
type _Headline = Expect<
  Equal<
    MessageParams<"Hello {name}, you have {count, plural, one {# item} other {# items}}">,
    { name: string; count: number }
  >
>;
type _NoParams = Expect<Equal<MessageParams<"Just text">, {}>>;
type _NoParamsFlag = Expect<Equal<HasNoParams<"Just text">, true>>;
type _HasParamsFlag = Expect<Equal<HasNoParams<"{x}">, false>>;
type _Whitespace = Expect<Equal<MessageParams<"{  spaced  }">, { spaced: string }>>;

// Typed formats.
type _Number = Expect<Equal<MessageParams<"{n, number}">, { n: number }>>;
type _NumberStyle = Expect<Equal<MessageParams<"{n, number, ::currency/USD}">, { n: number }>>;
type _NumberThenText = Expect<
  Equal<MessageParams<"{n, number} and {m, date}, done">, { n: number; m: Date | number }>
>;
type _DateTime = Expect<
  Equal<MessageParams<"{d, date, short} {t, time}">, { d: Date | number; t: Date | number }>
>;
type _Ordinal = Expect<
  Equal<
    MessageParams<"{p, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}">,
    { p: number }
  >
>;
type _Offset = Expect<
  Equal<
    MessageParams<"{n, plural, offset:1 =0 {none} one {{who}} other {{who} +#}}">,
    { n: number; who: string }
  >
>;

// Select keys become a literal union.
type _Select = Expect<
  Equal<
    MessageParams<"{g, select, male {He} female {She} other {They}}">,
    { g: "male" | "female" | "other" }
  >
>;
type _NestedSelect = Expect<
  Equal<
    MessageParams<"{a, select, x {{b, select, p {{c}} other {q}}} other {z}}">,
    { a: "x" | "other"; b: "p" | "other"; c: string }
  >
>;

// Typed occurrences win over plain ones; incompatible ones become never.
type _TypedWins = Expect<Equal<MessageParams<"{n} {n, number}">, { n: number }>>;
type _Conflict = Expect<
  Equal<MessageParams<"{v, select, a {x} other {y}} {v, number}">, { v: never }>
>;

// Apostrophe quoting hides braces from the parser.
type _Quoted = Expect<Equal<MessageParams<"It''s '{not}' {real}">, { real: string }>>;
type _QuotedInCase = Expect<Equal<MessageParams<"{n, plural, other {'{x}' #}}">, { n: number }>>;

// Widened strings fall back to an open record.
type _Widened = Expect<Equal<MessageParams<string>, Record<string, unknown>>>;

// Occurrences are exposed for merging.
type _Occurrences = Expect<
  Equal<
    MessageArguments<"{a} {b, number}">,
    readonly ["a", string, false] | readonly ["b", number, true]
  >
>;

// formatMessage checks params against the literal.
formatMessage("en", "Hi {name}", { name: "Ada" });
// @ts-expect-error missing parameter.
formatMessage("en", "Hi {name}", {});
// @ts-expect-error misspelled parameter.
formatMessage("en", "Hi {name}", { nmae: "Ada" });
// @ts-expect-error plural counts are numbers.
formatMessage("en", "{n, plural, other {#}}", { n: "2" });
// @ts-expect-error select values must be declared keys.
formatMessage("en", "{g, select, a {A} other {O}}", { g: "b" });
formatMessage("en", "{d, date}", { d: new Date() });
formatMessage("en", "{d, date}", { d: 0 });
