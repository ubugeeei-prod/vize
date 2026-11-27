# Chat behavior contract

Normative state × input → outcome table for `src/components/chat/chat.vue`,
`src/components/chat/chat-message.vue`, and `src/components/chat/chat-composer.vue`.
Every row is proven by `src/components/chat/tests/behavior.test.ts`.

| #   | State              | Input       | Outcome                                                     | Proven by                                 |
| --- | ------------------ | ----------- | ----------------------------------------------------------- | ----------------------------------------- |
| H1  | log with a message | render      | `role="log"`; the message is an article named by its author | `renders a log of authored messages`      |
| H2  | empty composer     | render      | the send button is disabled                                 | `sends trimmed text and clears the draft` |
| H3  | draft `  hi  `     | Enter       | one `send` with `hi`; the field is cleared                  | `sends trimmed text and clears the draft` |
| H4  | draft with text    | Shift+Enter | no `send`                                                   | `keeps Shift+Enter as a newline`          |
| H5  | disabled composer  | Enter       | no `send`                                                   | `a disabled composer does not send`       |
