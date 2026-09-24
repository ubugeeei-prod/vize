import type {
  SegmentedFieldExpose,
  SegmentedFieldSlotState,
} from "../date-field/date-field-types.ts";
import type { HourCycle, PlainTime, TimeGranularity } from "./plain-time.ts";

/** State exposed to the TimeField default slot. */
export interface TimeFieldSlotState extends SegmentedFieldSlotState<PlainTime> {
  /** Resolved hour clock. */
  readonly hourCycle: HourCycle;

  /** Smallest edited unit. */
  readonly granularity: TimeGranularity;
}

/** Public instance exposed by TimeField. */
export interface TimeFieldExpose extends SegmentedFieldExpose<PlainTime> {
  /** Resolved hour clock. */
  readonly hourCycle: HourCycle;

  /** Smallest edited unit. */
  readonly granularity: TimeGranularity;
}
