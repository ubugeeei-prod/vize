/** Compile-only assertions for the `use-event-source` type contracts. */

import { useEventSource } from "./use-event-source.ts";
import type { EventSourceFailure, EventSourceStatus } from "./use-event-source.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const text = useEventSource("/sse");
type _DefaultMessageIsText = Expect<Equal<typeof text.data.value, string | undefined>>;
type _DefaultEventName = Expect<Equal<typeof text.event.value, "message" | undefined>>;
type _StatusIsClosed = Expect<Equal<typeof text.status.value, EventSourceStatus>>;
type _ErrorIsTyped = Expect<Equal<typeof text.error.value, EventSourceFailure | undefined>>;

const feed = useEventSource<{ price: number; notice: string }>("/feed", {
  events: ["price", "notice"],
  parse: { price: (raw) => Number(raw) },
});
type _DataIsUnion = Expect<Equal<typeof feed.data.value, number | string | undefined>>;
feed.on("price", (price) => {
  type _HandlerDataIsNarrowed = Expect<Equal<typeof price, number>>;
});
const latest = feed.message.value;
if (latest?.event === "price") {
  type _MessageIsDiscriminated = Expect<Equal<typeof latest.data, number>>;
}

// @ts-expect-error non-text events require a parser.
useEventSource<{ price: number }>("/feed", { events: ["price"] });

// @ts-expect-error a parse map is required for non-text events even without options.
useEventSource<{ price: number }>("/feed");

// @ts-expect-error only declared event names can be subscribed.
useEventSource<{ message: string }>("/sse", { events: ["other"] });

// @ts-expect-error handlers receive the event's data type.
feed.on("notice", (notice: number) => notice);
