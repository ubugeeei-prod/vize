<!-- Support thread with one message and a composer. -->
<script setup lang="ts">
import { ref } from "vue";

import { Chat, ChatComposer, ChatMessage } from "../index.ts";

const messages = ref([{ author: "Ada", text: "The build is green." }]);

function reply(text: string): void {
  messages.value = [...messages.value, { author: "You", text }];
}
</script>

<template>
  <Chat label="Support">
    <ChatMessage v-for="message in messages" :key="message.text" :author="message.author">
      {{ message.text }}
    </ChatMessage>
    <ChatComposer @send="reply" />
  </Chat>
</template>
