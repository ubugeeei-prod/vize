import { Segmented, SegmentedItem } from "../index.ts";
import type { SegmentedItemProps, SegmentedOrientation, SegmentedProps } from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _OrientationStaysClosed = Expect<Equal<SegmentedOrientation, "horizontal" | "vertical">>;
type _GroupPropsFeedTheSfc = Expect<
  SegmentedProps extends InstanceType<typeof Segmented>["$props"] ? true : false
>;
type _ItemPropsFeedTheSfc = Expect<
  SegmentedItemProps extends InstanceType<typeof SegmentedItem>["$props"] ? true : false
>;

export const group = { label: "Billing", modelValue: "day" } satisfies SegmentedProps;
export const item = { value: "day" } satisfies SegmentedItemProps;

// @ts-expect-error orientation is a closed public contract.
export const badOrientation: SegmentedProps = { label: "Billing", orientation: "grid" };
