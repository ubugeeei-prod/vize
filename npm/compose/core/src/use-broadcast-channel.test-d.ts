/** Compile-only assertions for the `use-broadcast-channel` type contracts. */

import { useBroadcastChannel } from "./use-broadcast-channel.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type Message = { readonly type: "login" } | { readonly type: "logout" };
const channel = useBroadcastChannel<Message>("auth");
type _DataIsTyped = Expect<Equal<typeof channel.data.value, Message | undefined>>;
type _PostIsTyped = Expect<Equal<Parameters<typeof channel.post>, [message: Message]>>;

const guarded = useBroadcastChannel("n", {
  validate: (data): data is number => typeof data === "number",
});
type _GuardInfersMessage = Expect<Equal<typeof guarded.data.value, number | undefined>>;

// @ts-expect-error posts must match the message type.
channel.post({ type: "unknown" });
// @ts-expect-error data is read-only.
channel.data.value = undefined;
