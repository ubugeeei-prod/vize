/** Compile-only assertions for the typed variants recipe API. */

import { cx, createCx, defineVariants } from "./variants.ts";
import type { ClassValue, VariantProps } from "./variants.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const button = defineVariants({
  base: "btn",
  variants: {
    intent: { primary: "btn-primary", danger: "btn-danger" },
    size: { sm: "text-sm", md: "text-md", lg: "text-lg" },
    disabled: { true: "opacity-50", false: "" },
  },
  compoundVariants: [{ intent: "danger", size: ["md", "lg"], class: "shadow" }],
  defaultVariants: { intent: "primary", size: "md" },
});

type ButtonProps = VariantProps<typeof button>;
type _IntentIsALiteralUnion = Expect<
  Equal<ButtonProps["intent"], "primary" | "danger" | undefined>
>;
type _SizeIsALiteralUnion = Expect<Equal<ButtonProps["size"], "sm" | "md" | "lg" | undefined>>;
type _TrueFalseKeysBecomeBoolean = Expect<Equal<ButtonProps["disabled"], boolean | undefined>>;
type _ClassIsNotAVariantProp = Expect<
  Equal<"class" extends keyof ButtonProps ? true : false, false>
>;
type _RecipeReturnsAString = Expect<Equal<ReturnType<typeof button>, string>>;
type _VariantKeysAreTyped = Expect<
  Equal<typeof button.variantKeys, readonly ("intent" | "size" | "disabled")[]>
>;

button({ intent: "danger", disabled: true, class: ["extra", { active: true }] });
button();
// @ts-expect-error unknown options are rejected.
button({ intent: "warning" });
// @ts-expect-error unknown variants are rejected.
button({ tone: "loud" });
// @ts-expect-error boolean variants take booleans.
button({ disabled: "yes" });

defineVariants({
  variants: { size: { sm: "a", lg: "b" } },
  // @ts-expect-error defaults must name an existing option.
  defaultVariants: { size: "xl" },
});
defineVariants({
  variants: { size: { sm: "a", lg: "b" } },
  // @ts-expect-error compound conditions must name existing options.
  compoundVariants: [{ size: "xl", class: "c" }],
});

export const card = defineVariants({
  slots: { header: "card-header", body: "card-body" },
  base: "card",
  variants: {
    tone: {
      plain: { header: "border-b" },
      loud: { base: "ring", body: "font-bold" },
    },
  },
  compoundVariants: [{ tone: "loud", class: { header: "uppercase" } }],
});
const cardSlots = card({ tone: "loud" });
type _SlotRecipeHasEverySlot = Expect<Equal<keyof typeof cardSlots, "header" | "body" | "base">>;
type _SlotFunctionsReturnStrings = Expect<Equal<ReturnType<typeof cardSlots.header>, string>>;
type _SlotKeysAreTyped = Expect<
  Equal<typeof card.slotKeys, readonly ("header" | "body" | "base")[]>
>;
cardSlots.body({ class: "p-4", tone: "plain" });
// @ts-expect-error unknown slots do not exist.
void cardSlots.footer;
// @ts-expect-error compound slot maps name existing slots.
defineVariants({
  slots: { header: "h" },
  variants: { tone: { plain: { header: "x" } } },
  compoundVariants: [{ tone: "plain", class: { footer: "y" } }],
});

export const responsiveStack = defineVariants({
  variants: { direction: { row: "flex-row", column: "flex-col" } },
  responsive: ["sm", "md", "lg"],
});
responsiveStack({ direction: { initial: "column", md: "row" } });
type _ResponsivePropsAcceptBreakpointMaps = Expect<
  Equal<
    VariantProps<typeof responsiveStack>["direction"],
    | "row"
    | "column"
    | ({ readonly initial?: "row" | "column" } & {
        readonly sm?: "row" | "column";
        readonly md?: "row" | "column";
        readonly lg?: "row" | "column";
      })
    | undefined
  >
>;
// @ts-expect-error unknown breakpoints are rejected.
responsiveStack({ direction: { xl: "row" } });
// @ts-expect-error non-responsive recipes reject breakpoint maps.
button({ size: { initial: "sm" } });

type _CxReturnsString = Expect<Equal<ReturnType<typeof cx>, string>>;
const merged = createCx((value) => value.toUpperCase());
merged("a", ["b", { c: true }]) satisfies string;
const _classInput: ClassValue = [1, 2n, null, { d: 0 }];
// @ts-expect-error class values do not accept functions.
cx(() => "a");
