export type { ChatComposerProps, ChatMessageProps, ChatProps } from "./types.ts";

/** Live log of a conversation. */
export { default as Chat } from "./chat.vue";

/** One authored message in a chat log. */
export { default as ChatMessage } from "./chat-message.vue";

/** Composer that sends trimmed text on Enter and keeps Shift+Enter for newlines. */
export { default as ChatComposer } from "./chat-composer.vue";
