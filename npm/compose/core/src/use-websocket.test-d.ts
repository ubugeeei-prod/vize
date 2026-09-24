/** Compile-only assertions for the `use-websocket` type contracts. */

import { useWebSocket } from "./use-websocket.ts";
import type {
  WebSocketFailure,
  WebSocketRawData,
  WebSocketSendData,
  WebSocketStatus,
} from "./use-websocket.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const raw = useWebSocket("ws://raw");
type _RawDataByDefault = Expect<Equal<typeof raw.data.value, WebSocketRawData | undefined>>;
type _RawSend = Expect<Equal<Parameters<typeof raw.send>, [message: WebSocketSendData]>>;
type _StatusIsClosed = Expect<Equal<typeof raw.status.value, WebSocketStatus>>;
type _ErrorIsTyped = Expect<Equal<typeof raw.error.value, WebSocketFailure | undefined>>;

interface Event {
  readonly kind: "tick";
}
interface Command {
  readonly op: "subscribe";
}
const typed = useWebSocket("ws://typed", {
  parse: (): Event => ({ kind: "tick" }),
  serialize: (command: Command) => JSON.stringify(command),
});
type _ParseInfersIncoming = Expect<Equal<typeof typed.data.value, Event | undefined>>;
typed.send({ op: "subscribe" });
// @ts-expect-error outgoing messages keep the serializer's type.
typed.send({ op: "unsubscribe" });

// @ts-expect-error a typed incoming message requires a parser.
useWebSocket<Event>("ws://missing-parse", {});

// @ts-expect-error non-sendable outgoing messages require a serializer.
useWebSocket<WebSocketRawData, Command>("ws://missing-serialize", {});

// @ts-expect-error status is read-only.
raw.status.value = "open";
