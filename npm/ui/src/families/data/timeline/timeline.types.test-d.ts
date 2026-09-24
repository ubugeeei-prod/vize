/** Compile-only assertions for the public Timeline contract. */

import {
  Timeline,
  TimelineConnector,
  TimelineContent,
  TimelineIndicator,
  TimelineItem,
  TimelineRoot,
  TimelineTime,
  type TimelineItemExpose,
  type TimelineItemSlotState,
  type TimelineItemStatus,
  type TimelineOrientation,
  type TimelineRootExpose,
  type TimelineSlotState,
} from "./timeline.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _StatusIsLiteral = Expect<Equal<TimelineItemStatus, "complete" | "current" | "upcoming">>;
type _OrientationIsLiteral = Expect<Equal<TimelineOrientation, "horizontal" | "vertical">>;
type _SlotValueIsNullable = Expect<Equal<TimelineSlotState["value"], string | null>>;
type _ItemStatusIsNullable = Expect<
  Equal<TimelineItemSlotState["status"], TimelineItemStatus | null>
>;
type _RootElement = Expect<Equal<TimelineRootExpose["element"], HTMLOListElement | null>>;
type _ItemElement = Expect<Equal<TimelineItemExpose["element"], HTMLLIElement | null>>;

const rootProps: InstanceType<typeof TimelineRoot>["$props"] = {
  ariaLabel: "History",
  orientation: "horizontal",
  reversed: true,
  value: "shipped",
};
const itemProps: InstanceType<typeof TimelineItem>["$props"] = {
  status: "complete",
  value: "ordered",
};
const timeProps: InstanceType<typeof TimelineTime>["$props"] = { datetime: new Date() };

const badItem: InstanceType<typeof TimelineItem>["$props"] = {
  // @ts-expect-error status is a closed union.
  status: "late",
};

// @ts-expect-error datetime is required.
const missingTime: InstanceType<typeof TimelineTime>["$props"] = {};

void Timeline;
void TimelineConnector;
void TimelineContent;
void TimelineIndicator;
void badItem;
void itemProps;
void missingTime;
void rootProps;
void timeProps;
