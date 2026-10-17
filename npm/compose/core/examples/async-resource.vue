<script setup lang="ts">
import { ref } from "vue";
import { useAsyncResource } from "@vizejs/composable/async-resource";

type Service = "standard" | "express" | "unavailable";
interface Quote {
  service: string;
  price: string;
}

const responses = ref(0);
const aborted = ref(0);
const lastExecution = ref("none");
const { data, error, status, pending, execute, cancel, reset } = useAsyncResource(
  ({ signal }, service: Service) =>
    new Promise<Quote>((resolve, reject) => {
      // Local sample data: replace this producer with fetch(url, { signal }) in an app.
      const onAbort = () => {
        clearTimeout(timer);
        aborted.value += 1;
        reject(signal.reason);
      };
      const timer = setTimeout(
        () => {
          signal.removeEventListener("abort", onAbort);
          responses.value += 1;
          if (service === "unavailable") reject(new Error("No courier serves this sample area."));
          else
            resolve({
              service: service === "standard" ? "Standard delivery" : "Express delivery",
              price: service === "standard" ? "€4.90" : "€9.90",
            });
        },
        service === "standard" ? 800 : 300,
      );
      if (signal.aborted) onAbort();
      else signal.addEventListener("abort", onAbort, { once: true });
    }),
);

async function request(service: Service): Promise<void> {
  // execute() resolves to success/error/cancelled/superseded; loader failures do not reject it.
  const result = await execute(service);
  lastExecution.value = result.status;
}
</script>

<template>
  <div class="composable-example">
    <p>Try delayed local delivery quotes. No request is sent to a server.</p>
    <div class="example-actions">
      <button type="button" @click="() => request('standard')">Standard</button>
      <button type="button" @click="() => request('express')">Express</button>
      <button type="button" @click="() => request('unavailable')">Unavailable area</button>
      <button type="button" :disabled="!pending" @click="() => cancel()">Cancel request</button>
      <button type="button" @click="() => reset()">Reset quote</button>
    </div>
    <output aria-live="polite">{{
      status === "pending"
        ? "Getting a quote…"
        : status === "cancelled"
          ? "Quote request cancelled."
          : status === "error"
            ? "Could not get a quote."
            : status === "success"
              ? "Quote ready."
              : "Choose a delivery service."
    }}</output>
    <p v-if="error instanceof Error" role="alert">{{ error.message }}</p>
    <section v-if="data" class="quote" aria-label="Current quote">
      <strong>{{ data.service }} · {{ data.price }}</strong>
      <p>The last successful quote stays visible until reset.</p>
    </section>
    <p v-else>No delivery quote yet.</p>
    <p>
      {{ responses }} {{ responses === 1 ? "response" : "responses" }} · {{ aborted }} cancelled
      {{ aborted === 1 ? "request" : "requests" }}
    </p>
    <p>Last execution result: {{ lastExecution }}</p>
  </div>
</template>

<style scoped>
.quote {
  border: 1px solid #888;
  border-radius: 6px;
  padding: 12px;
  & p {
    margin-bottom: 0;
  }
}
</style>
