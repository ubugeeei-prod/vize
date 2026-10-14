<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useWebAuthn } from "@vizejs/composable/use-web-authn";

const mounted = useMounted();
const { supported, isUserVerifyingPlatformAuthenticatorAvailable } = useWebAuthn();
const platform = ref<boolean>();

async function check() {
  platform.value = await isUserVerifyingPlatformAuthenticatorAvailable();
}
</script>

<template>
  <div class="composable-example">
    <p>Offer passkey sign-in only on devices with a built-in authenticator.</p>
    <button type="button" :disabled="!mounted || !supported" @click="check">
      Check for passkey support
    </button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "WebAuthn is not available in this browser."
          : platform === undefined
            ? "WebAuthn is available."
            : platform
              ? "This device can create a passkey."
              : "No built-in authenticator; offer a security key or password."
    }}</output>
  </div>
</template>
