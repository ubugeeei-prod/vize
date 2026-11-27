import { Statistic } from "../index.ts";
import type { StatisticNotation, StatisticProps } from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _NotationStaysClosed = Expect<Equal<StatisticNotation, "standard" | "compact">>;
type _PublicPropsFeedTheSfc = Expect<
  StatisticProps extends InstanceType<typeof Statistic>["$props"] ? true : false
>;

export const props = {
  title: "Revenue",
  value: 1280,
  notation: "compact",
  prefix: "$",
} satisfies StatisticProps;

// @ts-expect-error notation is a closed public contract.
export const badNotation: StatisticProps = { title: "Revenue", value: 1, notation: "scientific" };
