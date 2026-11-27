import { Descriptions, DescriptionsItem } from "../index.ts";
import type { DescriptionsItemProps, DescriptionsProps } from "../index.ts";

type Expect<Condition extends true> = Condition;

type _ListPropsFeedTheSfc = Expect<
  DescriptionsProps extends InstanceType<typeof Descriptions>["$props"] ? true : false
>;
type _ItemPropsFeedTheSfc = Expect<
  DescriptionsItemProps extends InstanceType<typeof DescriptionsItem>["$props"] ? true : false
>;

export const list = { label: "Invoice" } satisfies DescriptionsProps;
export const item = { label: "Status" } satisfies DescriptionsItemProps;

// @ts-expect-error the list name is required.
export const unnamed: DescriptionsProps = {};
