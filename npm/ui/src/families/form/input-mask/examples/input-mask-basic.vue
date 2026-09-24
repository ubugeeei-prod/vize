<!-- Phone number field that masks digits as the user types and reports completion. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { MaskedInput, type InputMaskResult } from "../input-mask.ts";

const phone = ref("");
const lastComplete = ref("");
const inputId = useId();
const hintId = useId();

function onComplete(result: InputMaskResult): void {
  lastComplete.value = result.masked;
}
</script>

<template>
  <div>
    <label :for="inputId">Phone number</label>
    <MaskedInput
      :id="inputId"
      v-model="phone"
      mask="(999) 999-9999"
      name="phone"
      autocomplete="tel-national"
      :aria-describedby="hintId"
      @complete="onComplete"
    />
    <p :id="hintId">Ten digits, area code first.</p>
    <output :for="inputId"
      >Digits: {{ phone || "none" }}. Last complete number: {{ lastComplete || "none" }}</output
    >
  </div>
</template>
