<script setup lang="ts">
import { useWebSocket } from "@vizejs/composable/use-websocket";

// The docs have no server, so this class echoes messages back. In your app, drop `host`
// and use your WebSocket server's URL.
class EchoServer extends EventTarget {
  constructor(_url: string) {
    super();
    queueMicrotask(() => this.dispatchEvent(new Event("open")));
  }

  send(data: unknown) {
    queueMicrotask(() =>
      this.dispatchEvent(new MessageEvent("message", { data: `Echo: ${String(data)}` })),
    );
  }

  close() {
    queueMicrotask(() => this.dispatchEvent(new CloseEvent("close", { code: 1000 })));
  }
}

const { status, data, open, close, send } = useWebSocket("wss://chat.example.com", {
  host: EchoServer,
  immediate: false,
  autoReconnect: false,
});
</script>

<template>
  <div class="composable-example">
    <p>Connect a chat to a WebSocket server and react to its connection state.</p>
    <div class="example-actions">
      <button type="button" :disabled="status !== 'closed'" @click="() => open()">Connect</button>
      <button type="button" :disabled="status !== 'open'" @click="() => send('Hello!')">
        Say hello
      </button>
      <button type="button" :disabled="status === 'closed'" @click="() => close()">
        Disconnect
      </button>
    </div>
    <output aria-live="polite">Status: {{ status }}{{ data ? ` · ${data}` : "" }}</output>
  </div>
</template>
