<script setup lang="ts">
import { useEventSource } from "@vizejs/composable/use-event-source";

// The docs have no server, so this class replays a build log. In your app, drop `host`
// and point the URL at an endpoint that streams `text/event-stream`.
class ReplayedBuildLog extends EventTarget {
  readyState = 0;

  constructor(_url: string) {
    super();
    queueMicrotask(() => {
      this.readyState = 1;
      this.dispatchEvent(new Event("open"));
    });
    ["Compiling 42 files", "Type checking", "Build finished"].forEach((line, index) => {
      setTimeout(
        () => {
          if (this.readyState === 1)
            this.dispatchEvent(new MessageEvent("message", { data: line }));
        },
        (index + 1) * 1000,
      );
    });
  }

  close() {
    this.readyState = 2;
  }
}

const { status, data, open, close } = useEventSource("/api/build-events", {
  host: ReplayedBuildLog,
  immediate: false,
  autoReconnect: false,
});
</script>

<template>
  <div class="composable-example">
    <p>Follow server-sent build events and show the connection state.</p>
    <div class="example-actions">
      <button type="button" :disabled="status !== 'closed'" @click="() => open()">Connect</button>
      <button type="button" :disabled="status === 'closed'" @click="() => close()">
        Disconnect
      </button>
    </div>
    <output aria-live="polite">Status: {{ status }}{{ data ? ` · ${data}` : "" }}</output>
  </div>
</template>
