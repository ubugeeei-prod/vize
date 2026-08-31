/** Compile-only assertions for the public locale contract. */

import type { ComputedRef } from "vue";

import {
  localeTextMatches,
  normalizeLocaleText,
  resolveCollator,
  resolveDateTimeFormatter,
  resolveDirection,
  resolveDisplayNames,
  resolveListFormatter,
  resolveLocale,
  resolveNumberFormatter,
  resolveRelativeTimeFormatter,
  resolveSearchCollator,
  type LocaleCollatorOptions,
  type LocaleDateTimeFormatterOptions,
  type LocaleDisplayNamesOptions,
  type LocaleDisplayNamesOptionsInput,
  type LocaleDisplayNamesType,
  type LocaleFormatterOptionsInput,
  type LocaleListFormatterOptions,
  type LocaleNumberFormatterOptions,
  type LocaleRelativeTimeFormatterOptions,
  type LocaleTextMatchMode,
  type LocaleTextMatchOptions,
  type DirectionPreference,
  type TextDirection,
  useCollator,
  useDateTimeFormatter,
  useDirection,
  useDisplayNames,
  useListFormatter,
  useLocale,
  useNumberFormatter,
  useRelativeTimeFormatter,
  useSearchCollator,
} from "./locale.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const directions: readonly TextDirection[] = ["ltr", "rtl"];
export const preferences: readonly DirectionPreference[] = ["auto", "ltr", "rtl"];
// @ts-expect-error vertical is not a writing direction.
export const invalidDirection: TextDirection = "vertical";

export const resolved: TextDirection = resolveDirection("auto", "en-US");
export const canonicalLocale: string = resolveLocale("ja-jp");
export const numberOptions = {
  currency: "JPY",
  style: "currency",
} satisfies LocaleNumberFormatterOptions;
export const numberOptionsInput = (() =>
  numberOptions) satisfies LocaleFormatterOptionsInput<LocaleNumberFormatterOptions>;
export const dateTimeOptions = {
  dateStyle: "medium",
  timeZone: "UTC",
} satisfies LocaleDateTimeFormatterOptions;
export const listOptions = {
  style: "short",
  type: "conjunction",
} satisfies LocaleListFormatterOptions;
export const relativeTimeOptions = {
  numeric: "auto",
  style: "narrow",
} satisfies LocaleRelativeTimeFormatterOptions;
export const displayNamesType: LocaleDisplayNamesType = "region";
export const displayNamesOptions = {
  style: "short",
  type: displayNamesType,
} satisfies LocaleDisplayNamesOptions;
export const displayNamesOptionsInput = (() =>
  displayNamesOptions) satisfies LocaleDisplayNamesOptionsInput;
export const collatorOptions = {
  sensitivity: "base",
  usage: "search",
} satisfies LocaleCollatorOptions;
export const numberFormatter: Intl.NumberFormat = resolveNumberFormatter("ja-JP", numberOptions);
export const reactiveNumberFormatter: ComputedRef<Intl.NumberFormat> = useNumberFormatter(
  () => numberOptions,
);
export const dateTimeFormatter: Intl.DateTimeFormat = resolveDateTimeFormatter(
  "ja-JP",
  dateTimeOptions,
);
export const reactiveDateTimeFormatter: ComputedRef<Intl.DateTimeFormat> = useDateTimeFormatter(
  () => dateTimeOptions,
);
export const listFormatter: Intl.ListFormat = resolveListFormatter("ja-JP", listOptions);
export const reactiveListFormatter: ComputedRef<Intl.ListFormat> = useListFormatter(
  () => listOptions,
);
export const relativeTimeFormatter: Intl.RelativeTimeFormat = resolveRelativeTimeFormatter(
  "ja-JP",
  relativeTimeOptions,
);
export const reactiveRelativeTimeFormatter: ComputedRef<Intl.RelativeTimeFormat> =
  useRelativeTimeFormatter(() => relativeTimeOptions);
export const displayNamesFormatter: Intl.DisplayNames = resolveDisplayNames(
  "ja-JP",
  displayNamesOptions,
);
export const reactiveDisplayNames: ComputedRef<Intl.DisplayNames> =
  useDisplayNames(displayNamesOptions);
export const collator: Intl.Collator = resolveCollator("ja-JP", collatorOptions);
export const searchCollator: Intl.Collator = resolveSearchCollator("ja-JP");
export const reactiveCollator: ComputedRef<Intl.Collator> = useCollator(() => collatorOptions);
export const reactiveSearchCollator: ComputedRef<Intl.Collator> = useSearchCollator();
export const normalizedText: string = normalizeLocaleText("  Cafe\u0301\nnoir  ");
export const textMatchMode: LocaleTextMatchMode = "contains";
export const textMatchOptions = {
  collator: searchCollator,
  match: textMatchMode,
} satisfies LocaleTextMatchOptions;
export const textMatches: boolean = localeTextMatches("Café noir", "cafe", {
  collator: searchCollator,
});

type _LocaleIsComputedString = Expect<Equal<ReturnType<typeof useLocale>, ComputedRef<string>>>;
type _DirectionIsComputed = Expect<
  Equal<ReturnType<typeof useDirection>, ComputedRef<TextDirection>>
>;
type _NumberFormatterIsComputed = Expect<
  Equal<ReturnType<typeof useNumberFormatter>, ComputedRef<Intl.NumberFormat>>
>;
type _DateTimeFormatterIsComputed = Expect<
  Equal<ReturnType<typeof useDateTimeFormatter>, ComputedRef<Intl.DateTimeFormat>>
>;
type _ListFormatterIsComputed = Expect<
  Equal<ReturnType<typeof useListFormatter>, ComputedRef<Intl.ListFormat>>
>;
type _RelativeTimeFormatterIsComputed = Expect<
  Equal<ReturnType<typeof useRelativeTimeFormatter>, ComputedRef<Intl.RelativeTimeFormat>>
>;
type _DisplayNamesIsComputed = Expect<
  Equal<ReturnType<typeof useDisplayNames>, ComputedRef<Intl.DisplayNames>>
>;
type _CollatorIsComputed = Expect<
  Equal<ReturnType<typeof useCollator>, ComputedRef<Intl.Collator>>
>;
type _SearchCollatorIsComputed = Expect<
  Equal<ReturnType<typeof useSearchCollator>, ComputedRef<Intl.Collator>>
>;

// @ts-expect-error auto is a preference, not a resolved direction.
export const unresolved: TextDirection = "auto";
// @ts-expect-error formatter styles are closed unions.
resolveNumberFormatter("en-US", { style: "money" });
// @ts-expect-error date styles are closed unions.
resolveDateTimeFormatter("en-US", { dateStyle: "tiny" });
// @ts-expect-error list formatter types are closed unions.
resolveListFormatter("en-US", { type: "sentence" });
// @ts-expect-error relative-time numeric policy is a closed union.
resolveRelativeTimeFormatter("en-US", { numeric: "sometimes" });
// @ts-expect-error display-name type is required.
resolveDisplayNames("en-US", {});
// @ts-expect-error display-name types are closed unions.
resolveDisplayNames("en-US", { type: "continent" });
// @ts-expect-error collator usage is a closed union.
resolveCollator("en-US", { usage: "lookup" });
// @ts-expect-error text match modes are closed unions.
localeTextMatches("Alpha", "a", { collator: searchCollator, match: "fuzzy" });
