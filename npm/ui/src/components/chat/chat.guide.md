## A log, a message, and a composer

`Chat` is `role="log"` with polite live updates. `ChatMessage` is an article
named by its author. `ChatComposer` trims the draft, ignores an empty send,
and treats Enter as send. Shift+Enter stays a newline.

```vue
<script setup lang="ts">
import { Chat, ChatComposer, ChatMessage } from "@vizejs/ui/chat";
</script>

<template>
  <Chat label="Support">
    <ChatMessage author="Ada">The build is green.</ChatMessage>
    <ChatComposer @send="reply" />
  </Chat>
</template>
```

The composer does not call a model. The application owns the thread and appends
the `send` payload.
