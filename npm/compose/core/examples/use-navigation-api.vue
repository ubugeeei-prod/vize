<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useNavigationApi } from "@vizejs/composable/use-navigation-api";

const { supported, currentEntry, canGoBack, navigate, back } = useNavigationApi<{ step: number }>({
  parseState: (raw) =>
    typeof raw === "object" && raw !== null && typeof (raw as { step?: unknown }).step === "number"
      ? (raw as { step: number })
      : undefined,
});

const mounted = useMounted();

function goToStep(step: number) {
  void navigate(`#step-${step}`, { state: { step } });
}
</script>

<template>
  <div class="composable-example">
    <p>Move through wizard steps as real history entries, so Back works as expected.</p>
    <div class="example-actions">
      <button type="button" @click="() => goToStep(2)">Go to step 2</button>
      <button type="button" @click="() => goToStep(3)">Go to step 3</button>
      <button type="button" :disabled="!canGoBack" @click="() => back()">Back</button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Reading history…"
        : supported
          ? `Current entry: ${currentEntry?.url?.split("#")[1] ?? "start"}`
          : "The Navigation API is not available."
    }}</output>
  </div>
</template>
