<script setup lang="ts">
import { Chat, ChatComposer, ChatMessage } from "../index.ts";
</script>

<art title="Chat" status="ready">
  <variant name="Default" default>
    <Chat label="Support">
      <ChatMessage author="Ada">The build is green.</ChatMessage>
      <ChatComposer />
    </Chat>
  </variant>
  <variant name="Disabled composer">
    <Chat label="Support">
      <ChatMessage author="Ada">Waiting for the next run.</ChatMessage>
      <ChatComposer disabled />
    </Chat>
  </variant>
</art>
