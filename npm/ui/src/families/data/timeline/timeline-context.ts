import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";
import type { TimelineItemStatus, TimelineOrientation } from "./timeline-types.ts";

/** Shared state for Timeline compound parts. */
export interface TimelineContextValue {
  readonly value: ComputedRef<string | null>;
  readonly orientation: ComputedRef<TimelineOrientation>;
  readonly registerItem: (input: {
    readonly key: string;
    readonly element: Readonly<ShallowRef<HTMLLIElement | null>>;
  }) => CollectionRegistration<string>;
  readonly getIndex: (key: string) => number;
  readonly getCount: () => number;
  readonly getStatus: (index: number, value: string | null) => TimelineItemStatus | null;
  readonly setValue: (key: string, value: string | null) => void;
}

export const timelineContext = createContext<TimelineContextValue>("Timeline");

/** Item-level state shared with indicator, connector, content, and time parts. */
export interface TimelineItemContextValue {
  readonly status: ComputedRef<TimelineItemStatus | null>;
  readonly index: ComputedRef<number>;
  readonly last: ComputedRef<boolean>;
  readonly value: ComputedRef<string | null>;
}

export const timelineItemContext = createContext<TimelineItemContextValue>("TimelineItem");
