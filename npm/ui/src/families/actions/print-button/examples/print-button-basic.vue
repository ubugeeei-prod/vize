<!-- Print button that opens the browser print dialog and reports its lifecycle state. -->
<script setup lang="ts">
import { ref } from "vue";

import { PrintButton } from "../print-button.ts";

const lastResult = ref("Not printed yet");

function onPrint(): void {
  lastResult.value = "Print dialog closed";
}

function onError(): void {
  lastResult.value = "Printing is unavailable";
}
</script>

<template>
  <div>
    <PrintButton
      idle-label="Print invoice"
      printing-label="Opening print dialog"
      @print="onPrint"
      @error="onError"
    />
    <output>{{ lastResult }}</output>
  </div>
</template>
