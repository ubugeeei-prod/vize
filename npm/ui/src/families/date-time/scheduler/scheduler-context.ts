import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  DragSourceRegistration,
  DropTargetRegistration,
} from "../../interaction/drag-and-drop/drag-and-drop.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import type { SchedulerEvent } from "./scheduler-layout.ts";
import type { SchedulerSlotState } from "./scheduler-types.ts";

/** Drag gesture kinds published by scheduler events. */
export type SchedulerDragKind = "move" | "resize";

/** Payload carried by scheduler drag sessions. */
export interface SchedulerDragData {
  readonly eventId: string;
}

/** Shared state and handlers published by SchedulerRoot. */
export interface SchedulerContextValue {
  readonly id: ComputedRef<string>;
  readonly headingId: ComputedRef<string>;
  readonly root: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly slotState: ComputedRef<SchedulerSlotState<unknown>>;
  readonly focusedIso: ComputedRef<string | null>;
  readonly focusedMinute: ComputedRef<number>;
  readonly windowStart: ComputedRef<number>;
  readonly windowEnd: ComputedRef<number>;
  readonly slotMinutes: ComputedRef<number>;
  readonly maxLanes: ComputedRef<number>;
  readonly eventLabel: (event: SchedulerEvent<unknown>) => string;
  readonly canNavigate: () => boolean;
  readonly navigate: (step: -1 | 1) => boolean;
  readonly goToToday: () => boolean;
  readonly onSlotKeydown: (date: PlainDate, minute: number, event: KeyboardEvent) => void;
  readonly onSlotActivate: (date: PlainDate, minute: number, event: Event) => void;
  readonly onDayKeydown: (date: PlainDate, event: KeyboardEvent) => void;
  readonly onDayActivate: (date: PlainDate, event: Event) => void;
  readonly onEventActivate: (eventId: string, event: Event) => void;
  /** Record where inside an event the pointer grabbed it, in CSS pixels from its top. */
  readonly onEventPointerDown: (offset: number) => void;
  readonly onEventKeydown: (eventId: string, mode: "time" | "row", event: KeyboardEvent) => void;
  readonly registerEventSource: (
    eventId: MaybeRefOrGetter<string>,
    kind: SchedulerDragKind,
    element: Readonly<ShallowRef<HTMLElement | null>>,
  ) => DragSourceRegistration;
  readonly registerDayTarget: (
    date: MaybeRefOrGetter<PlainDate>,
    mode: "time" | "day",
    element: Readonly<ShallowRef<HTMLElement | null>>,
  ) => DropTargetRegistration;
}

export const schedulerContext = createContext<SchedulerContextValue>("Scheduler");
