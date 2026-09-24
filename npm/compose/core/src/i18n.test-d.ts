/** Compile-only assertions for the typed i18n layer. */

import type { ComputedRef, ShallowRef } from "vue";

import { defineI18n, defineLocale, defineMessages, useI18n } from "./i18n.ts";
import type { MessageKey, MessageParamsFor, TranslateArguments } from "./i18n.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const messages = defineMessages({
  en: {
    greet: "Hello {name}",
    cart: { items: "{count, plural, one {# item} other {# items}}" },
    order: "{status, select, paid {Paid on {when, date}} other {Pending}}",
    plain: "Hi",
  },
  ja: {
    greet: "こんにちは {name}",
    cart: { items: "{count}個" },
    order: "{status, select, paid {{when, date}に支払い済み} other {保留中}}",
    plain: "やあ",
  },
});
type Catalogs = typeof messages;

// Keys are dotted paths shared by every locale.
type _Keys = Expect<Equal<MessageKey<Catalogs>, "greet" | "cart.items" | "order" | "plain">>;

// Parameters merge across locales; typed occurrences win.
type _Greet = Expect<Equal<MessageParamsFor<Catalogs, "greet">, { name: string }>>;
type _Cart = Expect<Equal<MessageParamsFor<Catalogs, "cart.items">, { count: number }>>;
type _Order = Expect<
  Equal<MessageParamsFor<Catalogs, "order">, { status: "paid" | "other"; when: Date | number }>
>;
type _Plain = Expect<Equal<MessageParamsFor<Catalogs, "plain">, {}>>;
type _OptionalParams = Expect<Equal<TranslateArguments<{}>, [params?: {}]>>;
type _RequiredParams = Expect<Equal<TranslateArguments<{ a: string }>, [params: { a: string }]>>;

// Catalog validation.
defineMessages({
  en: { a: "x", b: "y" },
  // @ts-expect-error every locale must define every key.
  ja: { a: "x" },
});
defineMessages({
  en: { nested: { a: "x" } },
  // @ts-expect-error nested keys are checked too.
  ja: { nested: {} },
});
defineMessages({
  // @ts-expect-error parameter types must be compatible across locales.
  en: { a: "{v, select, x {x} other {y}}" },
  // @ts-expect-error parameter types must be compatible across locales.
  ja: { a: "{v, number}" },
});
defineMessages({ en: { a: "{n}" }, ja: { a: "{n, number}" } });

// defineLocale validates lazy catalogs against the schema.
const fr = defineLocale<Catalogs>()({
  greet: "Salut {name}",
  cart: { items: "{count, plural, one {# article} other {# articles}}" },
  order: "{status, select, paid {Payé} other {En attente}}",
  plain: "Salut",
});
type _LiteralsKept = Expect<Equal<typeof fr.greet, "Salut {name}">>;
const goodLocale = { greet: "Salut {name}", cart: { items: "x" }, order: "x", plain: "x" } as const;
defineLocale<Catalogs>()(goodLocale);
const badLocale = { greet: "Salut {nom}", cart: { items: "x" }, order: "x", plain: "x" } as const;
// @ts-expect-error translations may not invent parameters.
defineLocale<Catalogs>()(badLocale);
// @ts-expect-error keys must match the schema.
defineLocale<Catalogs>()({ greet: "x", cart: { items: "x" }, order: "x" });

// Instances.
const definition = defineI18n({ messages, loaders: { fr: async () => ({ default: fr }) } });
const i18n = definition.create({ locale: "fr" });
type _Locale = Expect<Equal<typeof i18n.locale, Readonly<ShallowRef<"en" | "ja" | "fr">>>>;
type _Loaded = Expect<Equal<typeof i18n.loadedLocales, ComputedRef<("en" | "ja" | "fr")[]>>>;
i18n.t("greet", { name: "a" });
i18n.t("plain");
i18n.t("cart.items", { count: 2 });
i18n.t("order", { status: "paid", when: new Date() });
// @ts-expect-error parameters are required when the message has any.
i18n.t("greet");
// @ts-expect-error parameter values are typed.
i18n.t("cart.items", { count: "2" });
// @ts-expect-error select values are the declared case keys.
i18n.t("order", { status: "refunded", when: 0 });
// @ts-expect-error unknown keys are rejected.
i18n.t("missing");
// @ts-expect-error unknown locales are rejected.
void i18n.setLocale("de");
// @ts-expect-error unknown locales are rejected at creation.
definition.create({ locale: "de" });
// @ts-expect-error lazy loaders must return the full key shape.
defineI18n({ messages, loaders: { fr: async () => ({ greet: "x" }) } });
useI18n(definition).t("plain") satisfies string;
