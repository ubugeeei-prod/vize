<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { usePushSubscription } from "@vizejs/composable/use-push-subscription";

const mounted = useMounted();
// Pass your service worker registration, for example from useServiceWorker().
const { supported, subscription, subscribe, unsubscribe } = usePushSubscription({
  immediate: false,
});
const vapidPublicKey =
  "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U";
</script>

<template>
  <div class="composable-example">
    <p>Subscribe to schedule-change alerts sent by your server.</p>
    <div class="example-actions">
      <button
        type="button"
        :disabled="!mounted || !supported || subscription !== null"
        @click="() => subscribe({ applicationServerKey: vapidPublicKey, userVisibleOnly: true })"
      >
        Subscribe to alerts
      </button>
      <button type="button" :disabled="!subscription" @click="() => unsubscribe()">
        Unsubscribe
      </button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? subscription
            ? "Subscribed to alerts."
            : "Not subscribed."
          : "Push messages are not available in this browser."
    }}</output>
  </div>
</template>
