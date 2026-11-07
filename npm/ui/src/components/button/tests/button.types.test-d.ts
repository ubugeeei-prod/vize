import { Button } from "../index.ts";
import type {
  ButtonProps,
  ButtonSize,
  ButtonSlotProps,
  ButtonTone,
  ButtonVariant,
} from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type ComponentProps = InstanceType<typeof Button>["$props"];

type _VariantsStayClosed = Expect<Equal<ButtonVariant, "solid" | "outline" | "quiet">>;
type _TonesStayClosed = Expect<Equal<ButtonTone, "accent" | "neutral" | "danger">>;
type _SizesStayClosed = Expect<Equal<ButtonSize, "sm" | "md" | "lg">>;
type _PublicPropsFeedTheSfc = Expect<ButtonProps extends ComponentProps ? true : false>;
type _SlotKeepsAvailabilityState = Expect<Equal<ButtonSlotProps["unavailable"], boolean>>;

export const props = {
  size: "lg",
  tone: "danger",
  variant: "outline",
  type: "submit",
} satisfies ButtonProps;

// @ts-expect-error variants are a closed public contract.
export const badVariant: ButtonProps = { variant: "glass" };
// @ts-expect-error tones are semantic rather than arbitrary palette names.
export const badTone: ButtonProps = { tone: "purple" };
// @ts-expect-error sizes are a closed public contract.
export const badSize: ButtonProps = { size: "xl" };
