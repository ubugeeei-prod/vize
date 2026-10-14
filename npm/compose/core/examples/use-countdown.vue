<script setup lang="ts">
import { ref } from "vue";
import { useCountdown } from "@vizejs/composable/use-countdown";

const sent = ref(0);
const { remaining, isActive, start } = useCountdown(30);

function sendCode() {
  sent.value += 1;
  start();
}
</script>

<template>
  <div class="composable-example">
    <p>Let a reader request a new sign-in code, then wait before asking again.</p>
    <button type="button" :disabled="isActive" @click="sendCode">
      {{ sent === 0 ? "Send code" : "Resend code" }}
    </button>
    <output aria-live="polite">{{
      sent === 0
        ? "No code sent yet."
        : isActive
          ? `Code sent. Resend available in ${remaining} s.`
          : `Code sent ${sent} times. You can request another one.`
    }}</output>
  </div>
</template>
