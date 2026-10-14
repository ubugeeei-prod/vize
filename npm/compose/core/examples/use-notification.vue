<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useWebNotification } from "@vizejs/composable/use-notification";

const mounted = useMounted();
const { supported, permission, requestPermission, show } = useWebNotification();

async function remind() {
  if (permission.value !== "granted" && (await requestPermission()) !== "granted") return;
  show("Your talk starts in 10 minutes", { body: "Room A · Vize at Vue Fes Japan" });
}
</script>

<template>
  <div class="composable-example">
    <p>Remind the reader before their talk starts, after asking for permission.</p>
    <button
      type="button"
      :disabled="!mounted || !supported || permission === 'denied'"
      @click="remind"
    >
      Remind me
    </button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Notification permission: ${permission}`
          : "Notifications are not available in this browser."
    }}</output>
  </div>
</template>
