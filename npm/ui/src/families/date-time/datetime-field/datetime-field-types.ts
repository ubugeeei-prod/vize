import type {
  SegmentedFieldExpose,
  SegmentedFieldSlotState,
} from "../date-field/date-field-types.ts";
import type { HourCycle, TimeGranularity } from "../time-field/plain-time.ts";
import type { PlainDateTime } from "./plain-date-time.ts";

/** State exposed to the DateTimeField default slot. */
export interface DateTimeFieldSlotState extends SegmentedFieldSlotState<PlainDateTime> {
  /** Resolved hour clock. */
  readonly hourCycle: HourCycle;

  /** Smallest edited unit. */
  readonly granularity: TimeGranularity;
}

/** Public instance exposed by DateTimeField. */
export interface DateTimeFieldExpose extends SegmentedFieldExpose<PlainDateTime> {
  /** Resolved hour clock. */
  readonly hourCycle: HourCycle;

  /** Smallest edited unit. */
  readonly granularity: TimeGranularity;
}
