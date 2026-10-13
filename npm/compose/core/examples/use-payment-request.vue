<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { usePaymentRequest } from "@vizejs/composable/use-payment-request";

const mounted = useMounted();
const { supported, canMakePayment } = usePaymentRequest({
  methods: [{ supportedMethods: "https://google.com/pay" }],
  details: { total: { label: "Workshop ticket", amount: { currency: "EUR", value: "49.00" } } },
});
const available = ref<boolean>();

async function check() {
  available.value = await canMakePayment();
}
</script>

<template>
  <div class="composable-example">
    <p>Offer a browser payment sheet only when the reader can pay with it.</p>
    <button type="button" :disabled="!mounted || !supported" @click="check">
      Check payment options
    </button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "The Payment Request API is not available; show your regular checkout."
          : available === undefined
            ? "Payment Request is available."
            : available
              ? "The reader can pay with the browser's payment sheet."
              : "No saved payment method; show your regular checkout."
    }}</output>
  </div>
</template>
