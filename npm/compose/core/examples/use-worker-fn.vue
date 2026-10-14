<script setup lang="ts">
import { ref } from "vue";
import { useWorkerFn } from "@vizejs/composable/use-worker-fn";

function countPrimes(limit: number): number {
  let count = 0;
  for (let candidate = 2; candidate <= limit; candidate += 1) {
    let prime = true;
    for (let divisor = 2; divisor * divisor <= candidate; divisor += 1) {
      if (candidate % divisor === 0) {
        prime = false;
        break;
      }
    }
    if (prime) count += 1;
  }
  return count;
}

const { status, run } = useWorkerFn(countPrimes);
const result = ref<number>();

async function calculate() {
  result.value = await run(200_000);
}
</script>

<template>
  <div class="composable-example">
    <p>Run a heavy calculation in a worker so the page stays responsive.</p>
    <button type="button" :disabled="status === 'running'" @click="calculate">
      Count primes up to 200,000
    </button>
    <output aria-live="polite">{{
      result === undefined ? `Worker status: ${status}` : `${result} primes found in a worker`
    }}</output>
  </div>
</template>
