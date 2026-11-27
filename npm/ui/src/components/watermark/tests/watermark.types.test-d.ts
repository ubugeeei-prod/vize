import { Watermark } from "../index.ts";
import type { WatermarkProps } from "../index.ts";

type Expect<Condition extends true> = Condition;

type _PublicPropsFeedTheSfc = Expect<
  WatermarkProps extends InstanceType<typeof Watermark>["$props"] ? true : false
>;

export const props = {
  text: "DRAFT",
  opacity: 0.2,
  gap: 180,
  rotate: -24,
} satisfies WatermarkProps;

// @ts-expect-error opacity is a number.
export const badOpacity: WatermarkProps = { text: "DRAFT", opacity: "low" };
