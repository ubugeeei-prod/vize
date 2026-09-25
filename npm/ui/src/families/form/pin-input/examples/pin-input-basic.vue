<!-- Six-digit one-time code split across fields, bound with v-model and reporting completion. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { PinInput, PinInputField } from "../pin-input.ts";

const code = ref("");
const verified = ref("");
const labelId = useId();

function onComplete(value: string): void {
  verified.value = value;
}
</script>

<template>
  <div>
    <p :id="labelId">Enter the 6-digit code we sent to your phone</p>
    <PinInput
      v-slot="{ indexes }"
      v-model="code"
      :length="6"
      otp
      name="verificationCode"
      :aria-labelledby="labelId"
      @complete="onComplete"
    >
      <PinInputField v-for="index in indexes" :key="index" :index />
    </PinInput>
    <output>{{
      verified ? `Verifying ${verified}…` : `${code.length} of 6 digits entered`
    }}</output>
  </div>
</template>
