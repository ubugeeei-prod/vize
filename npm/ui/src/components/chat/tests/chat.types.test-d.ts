import { Chat, ChatComposer, ChatMessage } from "../index.ts";
import type { ChatComposerProps, ChatMessageProps, ChatProps } from "../index.ts";

type Expect<Condition extends true> = Condition;

type _LogPropsFeedTheSfc = Expect<
  ChatProps extends InstanceType<typeof Chat>["$props"] ? true : false
>;
type _MessagePropsFeedTheSfc = Expect<
  ChatMessageProps extends InstanceType<typeof ChatMessage>["$props"] ? true : false
>;
type _ComposerPropsFeedTheSfc = Expect<
  ChatComposerProps extends InstanceType<typeof ChatComposer>["$props"] ? true : false
>;

export const log = { label: "Support" } satisfies ChatProps;
export const message = { author: "Ada" } satisfies ChatMessageProps;
export const composer = { submitLabel: "Reply" } satisfies ChatComposerProps;

// @ts-expect-error a message needs an author.
export const unnamed: ChatMessageProps = {};
