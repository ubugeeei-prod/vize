/** Compile-only assertions for the public Scheduler contract. */

import {
  SchedulerRoot,
  layoutTimeGrid,
  type SchedulerEventChange,
  type SchedulerEventData,
  type SchedulerRootExpose,
  type SchedulerSlotState,
  type SchedulerView,
  type TimeGridPlacement,
} from "./scheduler.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Ticket {
  readonly priority: "high" | "low";
}

declare const api: SchedulerRootExpose<Ticket>;
declare const state: SchedulerSlotState<Ticket>;
declare const change: SchedulerEventChange<Ticket>;
declare const events: readonly SchedulerEventData<Ticket>[];

type _View = Expect<Equal<SchedulerView, "day" | "week" | "month">>;
type _Data = Expect<Equal<typeof change.event.data, Ticket>>;
type _Columns = Expect<
  Equal<(typeof state.columns)[number]["placements"][number]["event"]["data"], Ticket>
>;
type _Layout = Expect<
  Equal<ReturnType<typeof layoutTimeGrid<Ticket>>, readonly TimeGridPlacement<Ticket>[]>
>;
type _Navigate = Expect<Equal<typeof api.navigate, (step: -1 | 1) => boolean>>;

type SchedulerProps = NonNullable<Parameters<typeof SchedulerRoot<Ticket>>[0]>;
const props: SchedulerProps = {
  events,
  view: "month",
  dayStartHour: 8,
  "onEvent-move": (value: SchedulerEventChange<Ticket>) => value.event.data.priority,
};

// @ts-expect-error views are day, week, or month.
const badView: SchedulerView = "year";

void props;
void badView;
